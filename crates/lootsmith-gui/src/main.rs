//! Lootsmith GUI.
//!
//! A thin `egui` front end over `lootsmith-core`. Slice 1: open/browse saves
//! and the Asset Library dumps, edit the common scalars, list items, and save
//! (in place with a `.bak`, or Save As). Format logic lives entirely in the
//! library.
//!
//! Run args:
//!   lootsmith-gui [--open <save.sav>] [--dumps <Asset Library Manager.json>]
//!   lootsmith-gui --selftest --open <save.sav>   (headless load/edit/save/reload)

use eframe::egui;
use lootsmith_core::{
    container, items, AssetGroup, AssetLibraryConfigs, AssetLibraryManager, Platform, SaveGame,
};
use std::path::PathBuf;
use std::process::ExitCode;

// ---------------------------------------------------------------------------
// Logging / diagnostics
// ---------------------------------------------------------------------------

fn log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(home).join(".cache/lootsmith");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("gui.log")
}

fn log_line(message: &str) {
    eprintln!("{message}");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        use std::io::Write;
        let _ = writeln!(file, "{message}");
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        log_line(&format!("PANIC: {info}"));
    }));
}

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CliArgs {
    open: Option<String>,
    dumps: Option<String>,
    selftest: bool,
}

fn parse_cli() -> CliArgs {
    let mut args = CliArgs::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--open" | "-o" => args.open = it.next(),
            "--dumps" | "-d" => args.dumps = it.next(),
            "--selftest" => args.selftest = true,
            other if args.open.is_none() && !other.starts_with('-') => {
                args.open = Some(other.to_string());
            }
            _ => {}
        }
    }
    args
}

fn main() -> ExitCode {
    install_panic_hook();
    let args = parse_cli();
    log_line(&format!("lootsmith-gui start: open={:?}", args.open));

    if args.selftest {
        return run_selftest(args.open.as_deref());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 660.0])
            .with_title("Lootsmith"),
        ..Default::default()
    };
    let app = LootsmithApp::new(args);
    match eframe::run_native(
        "Lootsmith",
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log_line(&format!("eframe error: {e}"));
            log_line("If the window failed to open, run with `env -u WAYLAND_DISPLAY` to force X11.");
            ExitCode::FAILURE
        }
    }
}

fn run_selftest(open: Option<&str>) -> ExitCode {
    let Some(path) = open else {
        log_line("selftest: --open <save.sav> is required");
        return ExitCode::from(2);
    };

    let run = (|| -> Result<String, String> {
        let data = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        let save = container::unpack(&data, Platform::PC).map_err(|e| format!("unpack: {e}"))?;
        if !save.crc32_valid() {
            return Err("input crc32 mismatch".into());
        }
        let mut save_game = save.save_game().map_err(|e| format!("protobuf: {e}"))?;

        let before = save_game.exp_level().unwrap_or(0);
        let after = before + 1;
        save_game.set_i32(2, after);

        let out = std::env::temp_dir().join("lootsmith-selftest.sav");
        let bytes = container::serialize(&save_game.to_bytes(), Platform::PC)
            .map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(&out, &bytes).map_err(|e| format!("write: {e}"))?;

        let reopen = container::unpack(&bytes, Platform::PC).map_err(|e| format!("reopen: {e}"))?;
        if !reopen.crc32_valid() || !reopen.proto_stable() || !reopen.huffman_stable() {
            return Err("reopen integrity failed".into());
        }
        let sg2 = reopen.save_game().map_err(|e| format!("reparse: {e}"))?;
        if sg2.exp_level() != Some(after) {
            return Err(format!("level did not persist: {:?}", sg2.exp_level()));
        }
        Ok(format!(
            "selftest OK: level {before} -> {after}, wrote {} bytes to {}",
            bytes.len(),
            out.display()
        ))
    })();

    match run {
        Ok(msg) => {
            log_line(&msg);
            ExitCode::SUCCESS
        }
        Err(e) => {
            log_line(&format!("selftest FAILED: {e}"));
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct Form {
    level: i32,
    exp_points: i32,
    general_skill_points: i32,
    specialist_skill_points: i32,
    playthroughs: i32,
    currency: i32,
    golden_keys: i32,
    bank_slots: i32,
}

struct LootsmithApp {
    path: String,
    dumps: String,
    manager: Option<AssetLibraryManager>,
    save_game: Option<SaveGame>,
    form: Form,
    items: Vec<String>,
    status: String,
    save_as: String,
    loaded_once: bool,
}

impl LootsmithApp {
    fn new(args: CliArgs) -> Self {
        let mut app = Self {
            path: args.open.unwrap_or_default(),
            dumps: args.dumps.unwrap_or_default(),
            manager: None,
            save_game: None,
            form: Form::default(),
            items: Vec::new(),
            status: "Enter a save path and click Load (or Browse).".to_string(),
            save_as: String::new(),
            loaded_once: false,
        };
        if !app.dumps.trim().is_empty() {
            app.load_dumps();
        }
        app
    }

    fn configs(&self) -> AssetLibraryConfigs {
        self.manager
            .as_ref()
            .map(|m| m.configs.clone())
            .unwrap_or_else(AssetLibraryConfigs::pc_default)
    }

    fn load(&mut self) {
        if self.path.trim().is_empty() {
            self.status = "Enter a save path (or click Browse).".to_string();
            return;
        }
        let data = match std::fs::read(self.path.trim()) {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("read error: {e}");
                return;
            }
        };
        let save = match container::unpack(&data, Platform::PC) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("parse error: {e}");
                return;
            }
        };
        let save_game = match save.save_game() {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("protobuf error: {e}");
                return;
            }
        };
        self.form = Form {
            level: save_game.exp_level().unwrap_or(1),
            exp_points: save_game.exp_points().unwrap_or(0),
            general_skill_points: save_game.general_skill_points().unwrap_or(0),
            specialist_skill_points: save_game.specialist_skill_points().unwrap_or(0),
            playthroughs: save_game.playthroughs_completed().unwrap_or(0),
            currency: save_game.currency_on_hand().first().copied().unwrap_or(0),
            golden_keys: save_game.num_golden_keys_notified().unwrap_or(0),
            bank_slots: save_game.max_bank_slots().unwrap_or(0),
        };
        self.save_game = Some(save_game);
        self.status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        log_line(&self.status);
        self.refresh_items();
    }

    fn load_dumps(&mut self) {
        let path = if self.dumps.trim().is_empty() {
            std::env::var("LOOTSMITH_ASSET_LIBRARY").unwrap_or_default()
        } else {
            self.dumps.clone()
        };
        if path.trim().is_empty() {
            self.manager = None;
            return;
        }
        match AssetLibraryManager::from_path(std::path::Path::new(path.trim())) {
            Ok(m) => {
                self.manager = Some(m);
                self.status = format!("loaded asset library from {path}");
            }
            Err(e) => {
                self.manager = None;
                self.status = format!("asset library error: {e}");
            }
        }
        self.refresh_items();
    }

    fn refresh_items(&mut self) {
        let mut lines = Vec::new();
        if let Some(save_game) = &self.save_game {
            let configs = self.configs();
            for (index, entry) in save_game.packed_entries().iter().enumerate() {
                match items::unpack(&entry.serial, &configs) {
                    Ok(decoded) => {
                        let kind = if decoded.is_weapon { "weapon" } else { "item" };
                        let balance = match (&self.manager, &decoded.slot) {
                            (Some(m), items::PackedSlot::Weapon(w)) => m.lookup_asset(
                                AssetGroup::BalanceDefs,
                                decoded.asset_library_set_id,
                                w.balance.asset_index,
                                w.balance.sublibrary_index,
                                w.balance.use_set_id,
                            ),
                            (Some(m), items::PackedSlot::Item(i)) => m.lookup_asset(
                                AssetGroup::BalanceDefs,
                                decoded.asset_library_set_id,
                                i.balance.asset_index,
                                i.balance.sublibrary_index,
                                i.balance.use_set_id,
                            ),
                            _ => Ok(format!("({} bytes)", entry.serial.len())),
                        }
                        .unwrap_or_else(|e| format!("<{e}>"));
                        lines.push(format!(
                            "#{} {kind} id={} balance={balance}",
                            index + 1,
                            decoded.unique_id
                        ));
                    }
                    Err(e) => lines.push(format!("#{} decode error: {e}", index + 1)),
                }
            }
        }
        self.items = lines;
    }

    fn apply_form(&mut self) -> Option<Vec<u8>> {
        let mut save_game = self.save_game.clone()?;
        save_game.set_i32(2, self.form.level);
        save_game.set_i32(3, self.form.exp_points);
        save_game.set_i32(4, self.form.general_skill_points);
        save_game.set_i32(5, self.form.specialist_skill_points);
        save_game.set_i32(7, self.form.playthroughs);
        let mut currency = save_game.get_int_list(6);
        if currency.is_empty() {
            currency.push(self.form.currency);
        } else {
            currency[0] = self.form.currency;
        }
        save_game.set_int_list_packed(6, &currency);
        save_game.set_i32(48, self.form.golden_keys);
        save_game.set_i32(56, self.form.bank_slots);
        match container::serialize(&save_game.to_bytes(), Platform::PC) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                self.status = format!("serialize error: {e}");
                None
            }
        }
    }

    fn save_in_place(&mut self) {
        let Some(bytes) = self.apply_form() else {
            return;
        };
        if self.path.trim().is_empty() {
            self.status = "nothing to save to".into();
            return;
        }
        let backup = format!("{}.bak", self.path);
        if !std::path::Path::new(&backup).exists() {
            if let Err(e) = std::fs::copy(self.path.trim(), &backup) {
                self.status = format!("backup error: {e}");
                return;
            }
        }
        if let Err(e) = std::fs::write(self.path.trim(), &bytes) {
            self.status = format!("write error: {e}");
            return;
        }
        self.load();
        self.status = format!("saved {} bytes (backup {backup})", bytes.len());
        log_line(&self.status);
    }

    fn save_as(&mut self) {
        let target = if self.save_as.trim().is_empty() {
            self.status = "enter a Save As path first".into();
            return;
        } else {
            self.save_as.clone()
        };
        let Some(bytes) = self.apply_form() else {
            return;
        };
        if let Err(e) = std::fs::write(target.trim(), &bytes) {
            self.status = format!("write error: {e}");
            return;
        }
        self.status = format!("saved {} bytes to {target}", bytes.len());
        log_line(&self.status);
    }

    fn browse_save(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Borderlands 2 save")
            .add_filter("Borderlands 2 save", &["sav"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.path = path.display().to_string();
            self.load();
        }
    }

    fn browse_dumps(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Asset Library Manager.json")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            self.dumps = path.display().to_string();
            self.load_dumps();
        }
    }
}

impl eframe::App for LootsmithApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.loaded_once {
            self.loaded_once = true;
            if !self.path.trim().is_empty() {
                self.load();
            }
        }

        egui::Panel::top("inputs").show(ui, |ui| {
            egui::Grid::new("paths").num_columns(4).show(ui, |ui| {
                ui.label("Save file");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Browse…").clicked() {
                    self.browse_save();
                }
                if ui.button("Load").clicked() {
                    self.load();
                }
                ui.end_row();

                ui.label("Asset Library");
                ui.text_edit_singleline(&mut self.dumps);
                if ui.button("Browse…").clicked() {
                    self.browse_dumps();
                }
                if ui.button("Load dumps").clicked() {
                    self.load_dumps();
                }
                ui.end_row();
            });
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status));
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.save_game.is_none() {
                ui.heading("Lootsmith");
                ui.label("Open a Borderlands 2 save to begin.");
                ui.label("Tip: Load the Asset Library JSON too, so items show real names.");
                return;
            }

            egui::Grid::new("fields").num_columns(2).show(ui, |ui| {
                ui.label("Level");
                ui.add(egui::DragValue::new(&mut self.form.level));
                ui.end_row();
                ui.label("Experience");
                ui.add(egui::DragValue::new(&mut self.form.exp_points));
                ui.end_row();
                ui.label("General skill points");
                ui.add(egui::DragValue::new(&mut self.form.general_skill_points));
                ui.end_row();
                ui.label("Specialist skill points");
                ui.add(egui::DragValue::new(&mut self.form.specialist_skill_points));
                ui.end_row();
                ui.label("Playthroughs");
                ui.add(egui::DragValue::new(&mut self.form.playthroughs));
                ui.end_row();
                ui.label("Currency");
                ui.add(egui::DragValue::new(&mut self.form.currency));
                ui.end_row();
                ui.label("Golden keys");
                ui.add(egui::DragValue::new(&mut self.form.golden_keys));
                ui.end_row();
                ui.label("Bank slots");
                ui.add(egui::DragValue::new(&mut self.form.bank_slots));
                ui.end_row();
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save (backup + write in place)").clicked() {
                    self.save_in_place();
                }
                ui.label("Save As");
                ui.text_edit_singleline(&mut self.save_as);
                if ui.button("Save As").clicked() {
                    self.save_as();
                }
            });

            ui.separator();
            ui.heading(format!("Items ({})", self.items.len()));
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in &self.items {
                        ui.label(line);
                    }
                });
        });
    }
}
