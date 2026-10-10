//! Lootsmith GUI binary: native window, CLI args, logging, and headless selftest.

use eframe::egui;
use lootsmith_core::{container, Platform};
use lootsmith_gui::app::LootsmithApp;
use std::path::PathBuf;
use std::process::ExitCode;

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
    std::panic::set_hook(Box::new(|info| log_line(&format!("PANIC: {info}"))));
}

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
            .with_inner_size([1020.0, 720.0])
            .with_title("Lootsmith"),
        ..Default::default()
    };
    let app = LootsmithApp::new(args.open, args.dumps);
    match eframe::run_native("Lootsmith", options, Box::new(move |_cc| Ok(Box::new(app)))) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log_line(&format!("eframe error: {e}"));
            log_line("If the window failed to open, try `env -u WAYLAND_DISPLAY` to force X11.");
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
        save_game.set_i32(2, before + 1);
        let out = std::env::temp_dir().join("lootsmith-selftest.sav");
        let bytes = container::serialize(&save_game.to_bytes(), Platform::PC)
            .map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(&out, &bytes).map_err(|e| format!("write: {e}"))?;
        let reopen = container::unpack(&bytes, Platform::PC).map_err(|e| format!("reopen: {e}"))?;
        if !reopen.crc32_valid() || !reopen.proto_stable() || !reopen.huffman_stable() {
            return Err("reopen integrity failed".into());
        }
        let sg2 = reopen.save_game().map_err(|e| format!("reparse: {e}"))?;
        if sg2.exp_level() != Some(before + 1) {
            return Err(format!("level did not persist: {:?}", sg2.exp_level()));
        }
        Ok(format!(
            "selftest OK: level {before} -> {}, wrote {} bytes to {}",
            before + 1,
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
