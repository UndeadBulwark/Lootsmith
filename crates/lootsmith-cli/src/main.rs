//! Lootsmith CLI: `inspect`, `verify`, and `dump`.

use lootsmith_core::{
    container, items, mission_status_name, AssetGroup, AssetLibraryConfigs, AssetLibraryManager,
    Container, Customizations, GameDefs, Missions, Platform, TravelStations, MISSION_STATUSES,
    VEHICLE_FAMILIES,
};
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "lootsmith — a Borderlands 2 save editor (Rust port of Gibbed's tool)\n\
         \n\
         USAGE:\n  \
         lootsmith inspect <file.sav> [--platform <id>]\n  \
         lootsmith verify  <file.sav> [--platform <id>]\n  \
         lootsmith dump    <file.sav> [--platform <id>]\n  \
         lootsmith saves   [--platform <id>]\n  \
         lootsmith new     --out OUT [--class NAME] [--platform <id>]\n  \
         lootsmith json    <file.sav> [--out FILE] [--platform <id>]\n  \
         lootsmith unjson  <file.json> --out OUT [--platform <id>]\n  \
         lootsmith profile <profile.bin> [--golden-keys N] [--badass-rank N] [--badass-tokens N]\n  \
          \
                          [--unlock-customizations] [--out OUT]\n  \
         lootsmith items   <file.sav> [--platform <id>] [--container bag|bank]\n  \
         lootsmith ammo    <file.sav> [--platform <id>]\n  \
         lootsmith challenges <file.sav> [--platform <id>]\n  \
         lootsmith world     <file.sav> [--platform <id>]\n  \
         lootsmith sdus    <file.sav> [--platform <id>]\n  \
         lootsmith travel  <file.sav> [--stations FILE]\n  \
         lootsmith vehicles <file.sav> [--customizations FILE]\n  \
         lootsmith missions <file.sav> [--missions-json FILE]\n  \
         lootsmith edit    <file.sav> [--level N] [--currency N] [--golden-keys N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --item N --part SLOT --set NAME --dumps FILE [--container bag|bank]\n  \
         lootsmith edit    <file.sav> --skill N [--grade G] [--points P] [--out OUT]\n  \
         lootsmith edit    <file.sav> [--travel-add NAME] [--travel-remove NAME] [--travel-last NAME] [--out OUT]\n  \
         lootsmith edit    <file.sav> [--steering-mode N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --ammo N [--amount F] [--sdu N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --market NAME --sdu-level L [--max-sdus] --out OUT\n  \
         lootsmith edit    <file.sav> --challenge-stat ID --value V [--prestige N]\n  \
                          [--explore-all] [--clear-lockouts] [--clear-mementos]\n  \
                          [--complete-all-missions] [--reset-all-missions]\n  \
                          [--unlock-all-travel --stations FILE] --out OUT\n  \
         lootsmith edit    <file.sav> --game-stage REGION --stage N --out OUT\n  \
         lootsmith edit    <file.sav> [--backpack-slots S] [--bank-size S] --out OUT\n  \
         lootsmith edit    <file.sav> [--plot-mission N] [--active-mission N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --mission-playthrough P --mission-index I --mission-status NAME [--out OUT]\n\
         \n\
         Platforms: pc (default), x360, ps3, psvita, shield, switch\n\
         --dumps <Asset Library Manager.json> resolves asset names (or set LOOTSMITH_ASSET_LIBRARY).\n\
         --class accepts a full class id or: soldier/axton, siren/maya, mercenary/salvador/gunzerker,\n\
         assassin/zer0/zero, lilac/krieg/psycho, tulip/gaige/mechromancer.\n\
         --stations <Travel Stations.json> resolves teleporter names (or set LOOTSMITH_TRAVEL_STATIONS).\n\
         --customizations <Customizations.json> resolves vehicle skin names (or set LOOTSMITH_CUSTOMIZATIONS).\n\
         --missions-json <Missions.json> resolves mission names (or set LOOTSMITH_MISSIONS).\n\
         --container selects the backpack (default) or the bank/stash for --item.\n\
         edit writes in place with a <file>.bak backup unless --out is given."
    );
    ExitCode::from(2)
}

struct Args {
    command: String,
    file: String,
    platform: Platform,
    dumps: Option<String>,
    out: Option<String>,
    backup: bool,
    level: Option<i32>,
    exp_points: Option<i32>,
    general_skill_points: Option<i32>,
    specialist_skill_points: Option<i32>,
    playthroughs: Option<i32>,
    currency: Option<i32>,
    golden_keys: Option<i32>,
    bank_slots: Option<i32>,
    item: Option<usize>,
    part: Option<String>,
    set: Option<String>,
    skill: Option<usize>,
    grade: Option<i32>,
    points: Option<i32>,
    stations: Option<String>,
    customizations: Option<String>,
    missions_json: Option<String>,
    travel_add: Option<String>,
    travel_remove: Option<String>,
    travel_last: Option<String>,
    steering_mode: Option<i32>,
    plot_mission: Option<i32>,
    active_mission: Option<i32>,
    mission_playthrough: Option<usize>,
    mission_index: Option<usize>,
    mission_status: Option<String>,
    class: Option<String>,
    ammo: Option<usize>,
    amount: Option<f32>,
    sdu: Option<i32>,
    sdu_level: Option<i32>,
    market: Option<String>,
    challenge_stat: Option<String>,
    stage: Option<i32>,
    explore_all: bool,
    clear_lockouts: bool,
    clear_mementos: bool,
    complete_all_missions: bool,
    reset_all_missions: bool,
    unlock_all_travel: bool,
    value: Option<f32>,
    prestige: Option<i32>,
    backpack_slots: Option<i32>,
    bank_slots_flag: Option<i32>,
    max_sdus: bool,
    badass_rank: Option<i64>,
    badass_tokens: Option<i64>,
    unlock_customizations: bool,
    scope: Scope,
}

/// Which container(s) an item command operates on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    All,
    Backpack,
    Bank,
}

impl Scope {
    fn parse(value: &str) -> Option<Scope> {
        match value {
            "all" => Some(Scope::All),
            "backpack" | "backpack-items" | "pack" | "bag" | "items" => Some(Scope::Backpack),
            "bank" | "stash" => Some(Scope::Bank),
            _ => None,
        }
    }
}

fn parse_args() -> Option<Args> {
    let mut it = std::env::args().skip(1);
    let command = it.next()?;
    let file = if matches!(command.as_str(), "saves" | "new") {
        String::new()
    } else {
        it.next()?
    };
    let mut args = Args {
        command,
        file,
        platform: Platform::PC,
        dumps: None,
        out: None,
        backup: true,
        level: None,
        exp_points: None,
        general_skill_points: None,
        specialist_skill_points: None,
        playthroughs: None,
        currency: None,
        golden_keys: None,
        bank_slots: None,
        item: None,
        part: None,
        set: None,
        skill: None,
        grade: None,
        points: None,
        stations: None,
        customizations: None,
        missions_json: None,
        travel_add: None,
        travel_remove: None,
        travel_last: None,
        steering_mode: None,
        plot_mission: None,
        active_mission: None,
        mission_playthrough: None,
        mission_index: None,
        mission_status: None,
        class: None,
        ammo: None,
        amount: None,
        sdu: None,
        sdu_level: None,
        market: None,
        challenge_stat: None,
        stage: None,
        explore_all: false,
        clear_lockouts: false,
        clear_mementos: false,
        complete_all_missions: false,
        reset_all_missions: false,
        unlock_all_travel: false,
        value: None,
        prestige: None,
        backpack_slots: None,
        bank_slots_flag: None,
        max_sdus: false,
        badass_rank: None,
        badass_tokens: None,
        unlock_customizations: false,
        scope: Scope::All,
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--platform" | "-p" => args.platform = Platform::from_id(&it.next()?)?,
            "--dumps" | "-d" => args.dumps = Some(it.next()?),
            "--out" | "-o" => args.out = Some(it.next()?),
            "--no-backup" => args.backup = false,
            "--container" | "-c" => args.scope = Scope::parse(&it.next()?)?,
            "--level" => args.level = Some(it.next()?.parse().ok()?),
            "--exp-points" => args.exp_points = Some(it.next()?.parse().ok()?),
            "--general-skill-points" => args.general_skill_points = Some(it.next()?.parse().ok()?),
            "--specialist-skill-points" => {
                args.specialist_skill_points = Some(it.next()?.parse().ok()?)
            }
            "--playthroughs" => args.playthroughs = Some(it.next()?.parse().ok()?),
            "--currency" => args.currency = Some(it.next()?.parse().ok()?),
            "--golden-keys" => args.golden_keys = Some(it.next()?.parse().ok()?),
            "--bank-slots" => args.bank_slots = Some(it.next()?.parse().ok()?),
            "--item" => args.item = Some(it.next()?.parse().ok()?),
            "--part" => args.part = Some(it.next()?),
            "--set" => args.set = Some(it.next()?),
            "--skill" => args.skill = Some(it.next()?.parse().ok()?),
            "--grade" => args.grade = Some(it.next()?.parse().ok()?),
            "--points" => args.points = Some(it.next()?.parse().ok()?),
            "--stations" => args.stations = Some(it.next()?),
            "--customizations" => args.customizations = Some(it.next()?),
            "--missions-json" => args.missions_json = Some(it.next()?),
            "--travel-add" => args.travel_add = Some(it.next()?),
            "--travel-remove" => args.travel_remove = Some(it.next()?),
            "--travel-last" => args.travel_last = Some(it.next()?),
            "--steering-mode" => args.steering_mode = Some(it.next()?.parse().ok()?),
            "--plot-mission" => args.plot_mission = Some(it.next()?.parse().ok()?),
            "--active-mission" => args.active_mission = Some(it.next()?.parse().ok()?),
            "--mission-playthrough" => args.mission_playthrough = Some(it.next()?.parse().ok()?),
            "--mission-index" => args.mission_index = Some(it.next()?.parse().ok()?),
            "--mission-status" => args.mission_status = Some(it.next()?),
            "--class" => args.class = Some(it.next()?),
            "--ammo" => args.ammo = Some(it.next()?.parse().ok()?),
            "--amount" => args.amount = Some(it.next()?.parse().ok()?),
            "--sdu" => args.sdu = Some(it.next()?.parse().ok()?),
            "--sdu-level" => args.sdu_level = Some(it.next()?.parse().ok()?),
            "--market" => args.market = Some(it.next()?),
            "--challenge-stat" => args.challenge_stat = Some(it.next()?),
            "--value" => args.value = Some(it.next()?.parse().ok()?),
            "--prestige" => args.prestige = Some(it.next()?.parse().ok()?),
            "--stage" => args.stage = Some(it.next()?.parse().ok()?),
            "--explore-all" => args.explore_all = true,
            "--clear-lockouts" => args.clear_lockouts = true,
            "--clear-mementos" => args.clear_mementos = true,
            "--complete-all-missions" => args.complete_all_missions = true,
            "--reset-all-missions" => args.reset_all_missions = true,
            "--unlock-all-travel" => args.unlock_all_travel = true,
            "--backpack-slots" => args.backpack_slots = Some(it.next()?.parse().ok()?),
            "--bank-size" => args.bank_slots_flag = Some(it.next()?.parse().ok()?),
            "--max-sdus" => args.max_sdus = true,
            "--badass-rank" => args.badass_rank = Some(it.next()?.parse().ok()?),
            "--badass-tokens" => args.badass_tokens = Some(it.next()?.parse().ok()?),
            "--unlock-customizations" => args.unlock_customizations = true,
            _ => return None,
        }
    }
    Some(args)
}

/// Load the name-parts dumps (`Weapon/Item Name Parts` + `Items.json`) — from
/// `LOOTSMITH_NAME_PARTS`, or next to `--dumps` / the asset library env var.
/// Compose a display name from resolved prefix/title asset paths, using the
/// name-parts map; `None` when nothing maps (`#n/m` indices are skipped).
fn item_display_name(
    name_parts: &Option<lootsmith_core::NameParts>,
    assets: &[&str],
) -> Option<String> {
    let parts = name_parts.as_ref()?;
    let mut words = Vec::new();
    for asset in assets {
        if let Some(word) = parts.part_name(asset) {
            words.push(word);
        }
    }
    (!words.is_empty()).then(|| words.join(" "))
}

fn load_name_parts(args: &Args) -> Option<lootsmith_core::NameParts> {
    let dumps = args
        .dumps
        .clone()
        .or_else(|| std::env::var("LOOTSMITH_ASSET_LIBRARY").ok());
    if let Some(dumps) = dumps {
        let path = std::path::PathBuf::from(dumps);
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().map(<std::path::Path>::to_path_buf)?
        };
        if let Ok(parts) = lootsmith_core::NameParts::from_dir(&dir) {
            return Some(parts);
        }
    }
    if let Ok(path) = std::env::var("LOOTSMITH_NAME_PARTS") {
        if !path.trim().is_empty() {
            if let Ok(parts) = lootsmith_core::NameParts::from_path(std::path::Path::new(&path)) {
                return Some(parts);
            }
        }
    }
    None
}

/// The part-picker definitions next to the dumps file, else the bundled tables.
fn load_defs(dumps: &Option<String>) -> Option<GameDefs> {
    let dumps = dumps
        .clone()
        .or_else(|| std::env::var("LOOTSMITH_ASSET_LIBRARY").ok());
    if let Some(dumps) = dumps {
        let path = std::path::PathBuf::from(dumps);
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().map(<std::path::Path>::to_path_buf)?
        };
        if let Ok(defs) = GameDefs::from_dir(&dir) {
            if !defs.is_empty() {
                return Some(defs);
            }
        }
    }
    lootsmith_core::bundled::defs().ok()
}

fn load_manager(dumps: &Option<String>) -> Option<AssetLibraryManager> {
    let path = dumps
        .clone()
        .or_else(|| std::env::var("LOOTSMITH_ASSET_LIBRARY").ok())?;
    match AssetLibraryManager::from_path(std::path::Path::new(&path)) {
        Ok(m) => {
            eprintln!("loaded asset library manager from {path}");
            Some(m)
        }
        Err(e) => {
            eprintln!("warning: cannot load {path}: {e}");
            None
        }
    }
}

/// Resolve a class argument to its full id: short names, full ids both work.
/// Note the ids: Lilac = Krieg (Psycho), Tulip = Gaige (Mechromancer).
fn resolve_class(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "soldier" | "axton" | "commando" => "GD_Soldier.Character.CharClass_Soldier",
        "siren" | "maya" => "GD_Siren.Character.CharClass_Siren",
        "mercenary" | "salvador" | "sal" | "gunzerker" => {
            "GD_Mercenary.Character.CharClass_Mercenary"
        }
        "assassin" | "zer0" | "zero" => "GD_Assassin.Character.CharClass_Assassin",
        "lilac" | "krieg" | "psycho" => "GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass",
        "tulip" | "gaige" | "mechromancer" => {
            "GD_Tulip_Mechromancer.Character.CharClass_Mechromancer"
        }
        _ => name,
    }
    .to_string()
}

fn usage_for_family(family: &str) -> Option<&'static str> {
    VEHICLE_FAMILIES
        .iter()
        .find(|(_, f)| *f == family)
        .map(|(usage, _)| *usage)
}

fn load_customizations(args: &Args) -> Option<Customizations> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(path) = &args.customizations {
        candidates.push(std::path::PathBuf::from(path));
    }
    if let Ok(env) = std::env::var("LOOTSMITH_CUSTOMIZATIONS") {
        candidates.push(std::path::PathBuf::from(env));
    }
    if let Some(dumps) = &args.dumps {
        if let Some(dir) = std::path::Path::new(dumps).parent() {
            candidates.push(dir.join("Customizations.json"));
        }
    }
    for candidate in candidates {
        if candidate.exists() {
            if let Ok(customizations) = Customizations::from_path(&candidate) {
                eprintln!("loaded customizations from {}", candidate.display());
                return Some(customizations);
            }
        }
    }
    None
}

fn load_stations_file(args: &Args) -> Option<TravelStations> {
    let candidates: Vec<String> = {
        let mut out = Vec::new();
        if let Some(path) = &args.stations {
            out.push(path.clone());
        }
        if let Ok(env) = std::env::var("LOOTSMITH_TRAVEL_STATIONS") {
            out.push(env);
        }
        out
    };
    for candidate in candidates {
        match TravelStations::from_path(std::path::Path::new(&candidate)) {
            Ok(stations) => return Some(stations),
            Err(e) => eprintln!("warning: cannot load {candidate}: {e}"),
        }
    }
    None
}

fn parse_status(value: &str) -> Option<i32> {
    if let Ok(number) = value.parse::<i32>() {
        return Some(number);
    }
    MISSION_STATUSES
        .iter()
        .position(|name| name.eq_ignore_ascii_case(value))
        .map(|i| i as i32)
}

fn load_missions(args: &Args) -> Option<Missions> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(path) = &args.missions_json {
        candidates.push(std::path::PathBuf::from(path));
    }
    if let Ok(env) = std::env::var("LOOTSMITH_MISSIONS") {
        candidates.push(std::path::PathBuf::from(env));
    }
    if let Some(dumps) = &args.dumps {
        if let Some(dir) = std::path::Path::new(dumps).parent() {
            candidates.push(dir.join("Missions.json"));
        }
    }
    for candidate in candidates {
        if candidate.exists() {
            if let Ok(missions) = Missions::from_path(&candidate) {
                eprintln!("loaded missions from {}", candidate.display());
                return Some(missions);
            }
        }
    }
    None
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Some(a) => a,
        None => return usage(),
    };

    if args.command == "saves" {
        let saves = lootsmith_core::locate::find_saves();
        for (i, candidate) in saves.iter().enumerate() {
            println!(
                "#{:<3} {}  {}",
                i + 1,
                candidate.modified_text(),
                candidate.path.display()
            );
        }
        println!("total: {} save(s) found", saves.len());
        return ExitCode::SUCCESS;
    }

    if args.command == "new" {
        let Some(out) = args.out.clone() else {
            eprintln!("error: new requires --out <file.sav>");
            return ExitCode::from(2);
        };
        let class = args
            .class
            .as_deref()
            .map(resolve_class)
            .unwrap_or_else(|| "GD_Soldier.Character.CharClass_Soldier".to_string());
        let mut save = lootsmith_core::savegame::SaveGame::blank(&class);
        save.refresh_guid();
        let bytes = match container::serialize(&save.to_bytes(), args.platform) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };
        match std::fs::write(&out, &bytes) {
            Ok(()) => println!(
                "wrote blank {} save ({}): {} bytes → {}",
                args.platform.id(),
                class,
                bytes.len(),
                out
            ),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }

    if args.command == "unjson" {
        let Some(out) = args.out.clone() else {
            eprintln!("error: unjson requires --out <file.sav>");
            return ExitCode::from(2);
        };
        let text = match std::fs::read_to_string(&args.file) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", args.file);
                return ExitCode::FAILURE;
            }
        };
        let save = match lootsmith_core::savejson::save_from_json(&text) {
            Ok(save) => save,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };
        let bytes = match container::serialize(&save.to_bytes(), args.platform) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };
        match std::fs::write(&out, &bytes) {
            Ok(()) => println!("wrote {} ({} bytes)", out, bytes.len()),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }

    if args.command == "profile" {
        let data = match std::fs::read(&args.file) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", args.file);
                return ExitCode::FAILURE;
            }
        };
        let mut profile = match lootsmith_core::profile::Profile::parse(&data) {
            Ok(profile) => profile,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };
        let mut changed = false;
        if let Some(keys) = args.golden_keys {
            profile.set_golden_keys(keys as i64);
            changed = true;
        }
        if let Some(rank) = args.badass_rank {
            profile.set_badass_rank(rank.max(0) as u32);
            changed = true;
        }
        if let Some(tokens) = args.badass_tokens {
            profile.set_badass_tokens(tokens.max(0) as u32);
            changed = true;
        }
        if args.unlock_customizations {
            if let Err(e) = profile.unlock_all_customizations() {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            changed = true;
        }
        println!(
            "golden keys:  {:?}\nbadass rank:  {:?}\nb tokens:     {:?}\nall customs:  {}",
            profile.golden_keys(),
            profile.badass_rank(),
            profile.badass_tokens(),
            profile.all_customizations_unlocked()
        );
        if let Ok(stats) = profile.stats() {
            for (name, value) in lootsmith_core::STAT_NAMES.iter().zip(stats.iter()) {
                println!("{name:26} {value:.1}");
            }
        }
        if changed {
            let bytes = profile.to_bytes();
            let target = args.out.clone().unwrap_or_else(|| args.file.clone());
            if args.out.is_none()
                && args.backup
                && !std::path::Path::new(&format!("{target}.bak")).exists()
            {
                let _ = std::fs::copy(&target, format!("{target}.bak"));
            }
            if let Err(e) = std::fs::write(&target, &bytes) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            println!("wrote {target} ({} bytes)", bytes.len());
        }
        return ExitCode::SUCCESS;
    }

    let data = match std::fs::read(&args.file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", args.file);
            return ExitCode::FAILURE;
        }
    };

    let save = match container::unpack(&data, args.platform) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match args.command.as_str() {
        "inspect" => {
            let crc_ok = save.crc32_valid();
            let huff_ok = save.huffman_stable();
            println!("file:        {}", args.file);
            println!("platform:    {}", save.platform.id());
            println!("compression: {}", save.platform.compression_scheme().name());
            println!("endian:      {:?}", save.platform.endian());
            println!("file bytes:  {}", data.len());
            println!("inner bytes: {}", save.inner.len());
            println!("version:     {}", save.version);
            println!(
                "crc32:       {:#010x} ({})",
                save.crc32,
                if crc_ok { "ok" } else { "MISMATCH" }
            );
            println!("protobuf:    {} bytes", save.proto.len());
            println!(
                "huffman:     {}",
                if huff_ok { "stable" } else { "UNSTABLE" }
            );
            if crc_ok && huff_ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        "verify" => {
            let crc_ok = save.crc32_valid();
            let huff_ok = save.huffman_stable();
            if crc_ok && huff_ok {
                println!("ok: crc32 verified and huffman layer is stable");
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "failed: crc32={} huffman={}",
                    if crc_ok { "ok" } else { "mismatch" },
                    if huff_ok { "stable" } else { "unstable" }
                );
                ExitCode::FAILURE
            }
        }
        "json" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let text = lootsmith_core::savejson::save_to_json(&save_game);
            if let Some(out) = &args.out {
                if std::fs::write(out, text).is_err() {
                    eprintln!("error: cannot write {out}");
                    return ExitCode::FAILURE;
                }
                println!("wrote {out}");
            } else {
                println!("{text}");
            }
            ExitCode::SUCCESS
        }
        "dump" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            println!("# {} — {}", args.file, save.platform.id());
            println!(
                "player_class={:?} level={:?} exp={:?} skill_points={:?}/{:?} playthroughs={:?}",
                save_game.player_class(),
                save_game.exp_level(),
                save_game.exp_points(),
                save_game.general_skill_points(),
                save_game.specialist_skill_points(),
                save_game.playthroughs_completed(),
            );
            println!(
                "currency={:?} save_id={:?} play_time={:?}s last_saved={:?} golden_keys={:?} bank={:?}",
                save_game.currency_on_hand(),
                save_game.save_game_id(),
                save_game.total_play_time(),
                save_game.last_saved_date(),
                save_game.num_golden_keys_notified(),
                save_game.max_bank_slots(),
            );
            println!("--- fields ---");
            for line in save_game.describe() {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        "items" => {
            let manager = match args
                .dumps
                .clone()
                .or_else(|| std::env::var("LOOTSMITH_ASSET_LIBRARY").ok())
            {
                Some(path) => match AssetLibraryManager::from_path(std::path::Path::new(&path)) {
                    Ok(m) => {
                        eprintln!("loaded asset library manager from {path}");
                        Some(m)
                    }
                    Err(e) => {
                        eprintln!("warning: cannot load {path}: {e}");
                        None
                    }
                },
                None => None,
            };
            // Display names from the Name Parts dumps (optional).
            let name_parts = load_name_parts(&args);
            let configs = manager
                .as_ref()
                .map(|m| m.configs.clone())
                .unwrap_or_else(AssetLibraryConfigs::pc_default);

            let resolve = |group: AssetGroup,
                           set_id: i32,
                           r: items::PackedAssetReference|
             -> String {
                match &manager {
                    Some(m) => m
                        .lookup_asset(
                            group,
                            set_id,
                            r.asset_index,
                            r.sublibrary_index,
                            r.use_set_id,
                        )
                        .unwrap_or_else(|_| format!("#{}/{}", r.asset_index, r.sublibrary_index)),
                    None => format!("#{}/{}", r.asset_index, r.sublibrary_index),
                }
            };

            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            if manager.is_none() {
                eprintln!(
                    "(no dumps file: showing raw indices; pass --dumps <Asset Library Manager.json>)"
                );
            }

            let mut count = 0usize;
            let mut backpack = 0usize;
            let mut bank = 0usize;
            for entry in save_game.entries() {
                let is_bank = entry.container == Container::Bank;
                match (args.scope, is_bank) {
                    (Scope::Backpack, true) | (Scope::Bank, false) => continue,
                    _ => {}
                }
                if is_bank {
                    bank += 1;
                } else {
                    backpack += 1;
                }
                count += 1;

                let kind = match entry.container {
                    Container::BackpackItem => "item",
                    Container::BackpackWeapon => "weapon",
                    Container::Bank => "bank",
                };
                let meta = match entry.container {
                    Container::BackpackItem => {
                        format!(" qty={} equipped={}", entry.quantity, entry.equipped)
                    }
                    Container::BackpackWeapon => format!(" quick={}", entry.quick_slot),
                    Container::Bank => String::new(),
                };
                match items::unpack(&entry.serial, &configs) {
                    Ok(decoded) => {
                        let set = decoded.asset_library_set_id;
                        match &decoded.slot {
                            items::PackedSlot::Weapon(w) => {
                                let prefix = resolve(AssetGroup::WeaponParts, set, w.prefix_part);
                                let title = resolve(AssetGroup::WeaponParts, set, w.title_part);
                                println!(
                                    "{kind:>6} #{count} id={} balance={} mfr={} grade={} \
                                     stage={}{meta}",
                                    decoded.unique_id,
                                    resolve(AssetGroup::BalanceDefs, set, w.balance),
                                    resolve(AssetGroup::Manufacturers, set, w.manufacturer),
                                    w.manufacturer_grade_index,
                                    w.game_stage,
                                );
                                if let Some(name) =
                                    item_display_name(&name_parts, &[&prefix, &title])
                                {
                                    println!("        name={name}");
                                }
                                println!(
                                    "        type={} body={} grip={} barrel={} sight={} stock={} \
                                     elemental={} acc1={} acc2={} material={} prefix={} title={}",
                                    resolve(AssetGroup::WeaponTypes, set, w.weapon_type),
                                    resolve(AssetGroup::WeaponParts, set, w.body_part),
                                    resolve(AssetGroup::WeaponParts, set, w.grip_part),
                                    resolve(AssetGroup::WeaponParts, set, w.barrel_part),
                                    resolve(AssetGroup::WeaponParts, set, w.sight_part),
                                    resolve(AssetGroup::WeaponParts, set, w.stock_part),
                                    resolve(AssetGroup::WeaponParts, set, w.elemental_part),
                                    resolve(AssetGroup::WeaponParts, set, w.accessory1_part),
                                    resolve(AssetGroup::WeaponParts, set, w.accessory2_part),
                                    resolve(AssetGroup::WeaponParts, set, w.material_part),
                                    prefix,
                                    title,
                                );
                            }
                            items::PackedSlot::Item(it) => {
                                let prefix = resolve(AssetGroup::ItemParts, set, it.prefix_part);
                                let title = resolve(AssetGroup::ItemParts, set, it.title_part);
                                println!(
                                    "{kind:>6} #{count} id={} balance={} mfr={} grade={} \
                                     stage={}{meta}",
                                    decoded.unique_id,
                                    resolve(AssetGroup::BalanceDefs, set, it.balance),
                                    resolve(AssetGroup::Manufacturers, set, it.manufacturer),
                                    it.manufacturer_grade_index,
                                    it.game_stage,
                                );
                                if let Some(name) =
                                    item_display_name(&name_parts, &[&prefix, &title])
                                {
                                    println!("        name={name}");
                                }
                            }
                        }
                    }
                    Err(e) => println!("{kind:>6} #{count} decode error: {e}"),
                }
            }
            println!("total: {count} packed blobs (backpack {backpack}, bank {bank})");
            ExitCode::SUCCESS
        }
        "ammo" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let resources = save_game.resources();
            if resources.is_empty() {
                println!("(no ammo/resource pools in this save)");
            }
            for (i, pool) in resources.iter().enumerate() {
                println!(
                    "#{:<2} {:18} amount={:>7.1} sdu={} ({})",
                    i + 1,
                    pool.label(),
                    pool.amount,
                    pool.upgrade_level,
                    pool.pool
                );
            }
            println!("total: {} pools", resources.len());
            ExitCode::SUCCESS
        }
        "world" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let regions = save_game.region_game_stages();
            println!("region game stages ({}):", regions.len());
            for (i, region) in regions.iter().enumerate() {
                println!(
                    "#{:<2} {} stage={} playthrough={} dlc={}",
                    i + 1,
                    region.region,
                    region.game_stage,
                    region.playthrough_index,
                    region.dlc_package_id
                );
            }
            let (discovered, total) = {
                let discoveries = save_game.world_discoveries();
                let total = discoveries.len();
                (discoveries.iter().filter(|d| d.uncovered).count(), total)
            };
            println!("world discoveries: {discovered}/{total} uncovered");
            for (i, discovery) in save_game.world_discoveries().iter().enumerate() {
                println!(
                    "#{:<2} {} {}",
                    i + 1,
                    if discovery.uncovered { "x" } else { " " },
                    discovery.name
                );
            }
            let lockouts = save_game.lockouts();
            println!("lockouts ({}):", lockouts.len());
            for lockout in &lockouts {
                println!(
                    "  {} ({} = {})",
                    lockout.label(),
                    lockout.lockout,
                    lockout.time
                );
            }
            let expansions = save_game.dlc_expansions();
            println!("dlc expansions ({}):", expansions.len());
            for expansion in &expansions {
                println!(
                    "  tag={} data={} bytes",
                    expansion.tag,
                    expansion.data.len()
                );
            }
            let (weapons, items) = save_game.memento_counts();
            println!("mementos: weapons={weapons} items={items}");
            println!(
                "fully explored = {:?}",
                save_game
                    .message
                    .get_all(46)
                    .filter_map(|f| f.as_str().map(|s| s.into_owned()))
                    .collect::<Vec<_>>()
            );
            ExitCode::SUCCESS
        }
        "challenges" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            println!("prestiges = {:?}", save_game.num_challenge_prestiges());
            println!("level unlocks = {:?}", save_game.level_challenge_unlocks());
            match save_game.challenge_stats() {
                Ok(stats) => {
                    for entry in &stats.entries {
                        println!(
                            "challenge {}{}: total={} previous={}",
                            entry.id,
                            if entry.previous_value > 0 {
                                " (reset)"
                            } else {
                                ""
                            },
                            entry.total_value,
                            entry.previous_value
                        );
                    }
                    println!("total: {} challenge stat entries", stats.entries.len());
                }
                Err(e) => eprintln!("challenge stats: {e}"),
            }
            ExitCode::SUCCESS
        }
        "sdus" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let levels = save_game.black_market_upgrades();
            for (i, (name, level)) in lootsmith_core::savegame::BLACK_MARKET_KEYS
                .iter()
                .zip(levels.iter())
                .enumerate()
            {
                println!("#{} {:16} level={}", i + 1, name, level);
            }
            if let Some(slots) = save_game.inventory_slots() {
                println!(
                    "backpack max = {} slots; weapon-ready = {}; quick = {}",
                    slots.0, slots.1, slots.2
                );
            }
            println!("MaxBankSlots = {:?}", save_game.max_bank_slots());
            ExitCode::SUCCESS
        }
        "skills" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let skills = save_game.skills();
            for (i, skill) in skills.iter().enumerate() {
                println!(
                    "#{:<3} grade={} points={} slot={} {}",
                    i + 1,
                    skill.grade,
                    skill.grade_points,
                    skill.equipped_slot_index,
                    skill.skill
                );
            }
            println!("total: {} skills", skills.len());
            ExitCode::SUCCESS
        }
        "travel" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let stations = args
                .stations
                .clone()
                .or_else(|| std::env::var("LOOTSMITH_TRAVEL_STATIONS").ok())
                .map(|path| {
                    (
                        path.clone(),
                        TravelStations::from_path(std::path::Path::new(&path)),
                    )
                });
            let stations = match stations {
                Some((path, Ok(s))) => {
                    eprintln!("loaded travel stations from {path}");
                    Some(s)
                }
                Some((path, Err(e))) => {
                    eprintln!("warning: cannot load {path}: {e}");
                    None
                }
                None => None,
            };
            let label = |name: &str| match &stations {
                Some(s) => s.label_for(name),
                None => name.to_string(),
            };
            let visited = save_game.visited_teleporters();
            for (i, name) in visited.iter().enumerate() {
                println!("#{:<3} {}", i + 1, label(name));
            }
            println!(
                "last visited: {}",
                match save_game.last_visited_teleporter() {
                    Some(name) => label(&name),
                    None => "(none)".to_string(),
                }
            );
            println!("total: {} visited teleporters", visited.len());
            ExitCode::SUCCESS
        }
        "vehicles" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let customizations = load_customizations(&args);
            let label = |key: &str| match &customizations {
                Some(c) => c.label_for(key),
                None => key.to_string(),
            };
            let vehicles = save_game.vehicles();
            for vehicle in &vehicles {
                let usage = usage_for_family(&vehicle.family).unwrap_or("?");
                println!("{usage:<12} {}", vehicle.family);
                for (i, customization) in vehicle.customizations.iter().enumerate() {
                    println!("    slot {}: {}", i + 1, label(customization));
                }
            }
            if vehicles.is_empty() {
                println!("(no vehicle families set)");
            }
            println!("steering mode: {:?}", save_game.vehicle_steering_mode());
            println!("total: {} vehicle families set", vehicles.len());
            ExitCode::SUCCESS
        }
        "missions" => {
            let save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let missions = load_missions(&args);
            let label = |key: &str| match &missions {
                Some(m) => m.label_for(key),
                None => key.to_string(),
            };
            let name_for = |number: i32| match &missions {
                Some(m) => m
                    .label_for_number(number)
                    .unwrap_or_else(|| format!("(unknown {number})")),
                None => number.to_string(),
            };
            println!(
                "plot mission:   {}",
                save_game
                    .plot_mission_number()
                    .map(name_for)
                    .unwrap_or_else(|| "(none)".to_string())
            );
            println!(
                "active mission: {}",
                save_game
                    .active_mission_number()
                    .map(name_for)
                    .unwrap_or_else(|| "(none)".to_string())
            );
            let playthroughs = save_game.mission_playthroughs();
            for (pi, pt) in playthroughs.iter().enumerate() {
                println!(
                    "playthrough {pi} (number {:?}) active={} pending={} filtered={}",
                    pt.playthrough_number,
                    label(&pt.active_mission),
                    pt.pending_rewards,
                    pt.filtered_missions.len()
                );
                for (mi, mission) in pt.missions.iter().enumerate() {
                    println!(
                        "    [{mi:>3}] {:<26} {}",
                        mission_status_name(mission.status),
                        label(&mission.mission)
                    );
                }
            }
            let challenges = save_game.challenges();
            println!("challenges: {}", challenges.len());
            for (i, challenge) in challenges.iter().enumerate() {
                let dlc = if challenge.is_from_dlc {
                    format!(" (dlc {})", challenge.dlc_package_id)
                } else {
                    String::new()
                };
                println!("    [{i:>3}] {}{dlc}", challenge.challenge);
            }
            println!("prestiges: {:?}", save_game.num_challenge_prestiges());
            ExitCode::SUCCESS
        }
        "edit" => {
            let mut save_game = match save.save_game() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot parse protobuf: {e}");
                    return ExitCode::FAILURE;
                }
            };

            let mut changes = 0u32;
            if let Some(v) = args.level {
                save_game.set_i32(2, v);
                changes += 1;
            }
            if let Some(v) = args.exp_points {
                save_game.set_i32(3, v);
                changes += 1;
            }
            if let Some(v) = args.general_skill_points {
                save_game.set_i32(4, v);
                changes += 1;
            }
            if let Some(v) = args.specialist_skill_points {
                save_game.set_i32(5, v);
                changes += 1;
            }
            if let Some(v) = args.playthroughs {
                save_game.set_i32(7, v);
                changes += 1;
            }
            if let Some(v) = args.currency {
                let mut list = save_game.get_int_list(6);
                if list.is_empty() {
                    list.push(v);
                } else {
                    list[0] = v;
                }
                save_game.set_int_list_packed(6, &list);
                changes += 1;
            }
            if let Some(v) = args.golden_keys {
                save_game.set_i32(48, v);
                changes += 1;
            }
            if let Some(v) = args.bank_slots {
                save_game.set_i32(56, v);
                changes += 1;
            }

            if let (Some(index), Some(part), Some(name)) =
                (args.item, args.part.clone(), args.set.clone())
            {
                let Some(manager) = load_manager(&args.dumps) else {
                    eprintln!(
                        "error: --item/--part/--set needs --dumps <Asset Library Manager.json>"
                    );
                    return ExitCode::FAILURE;
                };
                let entries = match args.scope {
                    Scope::Bank => save_game.bank_entries(),
                    _ => save_game.packed_entries(),
                };
                if index == 0 || index > entries.len() {
                    eprintln!(
                        "error: item index {index} out of range (1..{})",
                        entries.len()
                    );
                    return ExitCode::FAILURE;
                }
                let entry = &entries[index - 1];
                let mut decoded = match items::unpack(&entry.serial, &manager.configs) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                // Warn (do not block) when the part is not one the item's
                // balance resolves for the slot.
                if let Some(defs) = load_defs(&args.dumps) {
                    let type_path =
                        lootsmith_core::defs::slot_asset_path(&manager, &decoded, "type");
                    let balance =
                        lootsmith_core::defs::slot_asset_path(&manager, &decoded, "balance")
                            .unwrap_or_default();
                    if !defs.is_legal_part(
                        decoded.is_weapon,
                        type_path.as_deref(),
                        &balance,
                        &part,
                        &name,
                    ) {
                        eprintln!(
                            "warning: {name} is not a listed {part} for this item's balance; \
                             writing anyway"
                        );
                    }
                }
                let group = match decoded.set_part(&manager, &part, &name) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let new_serial = match items::pack(&decoded, &manager.configs) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                if let Err(e) =
                    save_game.replace_serial(entry.field_number, entry.occurrence, &new_serial)
                {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
                println!(
                    "{} #{index}: set {part} = {name} ({})",
                    entry.container.label(),
                    group.key()
                );
                changes += 1;
            }

            if let Some(skill) = args.skill {
                let count = save_game.skills().len();
                if skill == 0 || skill > count {
                    eprintln!("error: skill index {skill} out of range (1..{count})");
                    return ExitCode::FAILURE;
                }
                if args.grade.is_none() && args.points.is_none() {
                    eprintln!("error: --skill needs --grade and/or --points");
                    return ExitCode::FAILURE;
                }
                if let Some(grade) = args.grade {
                    if let Err(e) = save_game.set_skill_grade(skill - 1, grade) {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                    changes += 1;
                }
                if let Some(points) = args.points {
                    if let Err(e) = save_game.set_skill_grade_points(skill - 1, points) {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                    changes += 1;
                }
                let now = save_game.skills().into_iter().nth(skill - 1).unwrap();
                println!(
                    "skill #{skill}: grade={} points={} ({})",
                    now.grade, now.grade_points, now.skill
                );
            }

            if let Some(name) = args.travel_add.clone() {
                if save_game.add_visited_teleporter(&name) {
                    println!("travel: added {name}");
                    changes += 1;
                } else {
                    eprintln!("note: {name} was already visited");
                }
            }
            if let Some(name) = args.travel_remove.clone() {
                if save_game.remove_visited_teleporter(&name) {
                    println!("travel: removed {name}");
                    changes += 1;
                } else {
                    eprintln!("note: {name} was not visited");
                }
            }
            if let Some(name) = args.travel_last.clone() {
                save_game.set_last_visited_teleporter(&name);
                println!("travel: last visited = {name}");
                changes += 1;
            }

            if let Some(mode) = args.steering_mode {
                save_game.set_vehicle_steering_mode(mode);
                println!("vehicle steering mode = {mode}");
                changes += 1;
            }
            if args.max_sdus {
                for (index, cap) in lootsmith_core::savegame::SDU_LEVEL_MAX.iter().enumerate() {
                    save_game.set_sdu(index, *cap).expect("max sdu");
                }
                println!("all SDUs maxed (ammo 7 / backpack 9 = 39 slots / bank 9 = 24 slots)");
                changes += 1;
            }
            if let Some(slots) = args.backpack_slots {
                let sdus = ((slots - 12) as f32 / 3.0).ceil().max(0.0) as i32;
                let slots = 12 + sdus * 3;
                save_game.set_sdu(7, sdus).expect("backpack sdu");
                println!("backpack = {slots} slots (SDU level {sdus})");
                changes += 1;
            }
            if let Some(slots) = args.bank_slots_flag {
                let sdus = ((slots - 6) as f32 / 2.0).ceil().clamp(0.0, 255.0) as i32;
                let slots = 6 + sdus * 2;
                save_game.set_sdu(8, sdus).expect("bank sdu");
                println!("bank = {slots} slots (SDU level {sdus})");
                changes += 1;
            }
            if let Some(name) = args.market.clone() {
                let Some(level) = args.sdu_level else {
                    eprintln!("error: --market NAME needs --sdu-level L");
                    return ExitCode::FAILURE;
                };
                let keys = lootsmith_core::savegame::BLACK_MARKET_KEYS;
                let position = keys.iter().position(|k| k.eq_ignore_ascii_case(&name));
                let index =
                    position.or_else(|| name.parse::<usize>().ok().map(|one| one.max(1) - 1));
                match index {
                    Some(index) if index < keys.len() => {
                        save_game.set_sdu(index, level).expect("sdu");
                        println!("{} SDU = level {level}", keys[index]);
                        changes += 1;
                    }
                    Some(_) => eprintln!("error: SDU index out of range (use 1..9)"),
                    None => eprintln!("error: unknown SDU '{name}' (use one of {keys:?} or 1..9)"),
                }
            }
            if args.explore_all {
                let changed = save_game.mark_all_discovered().expect("explore");
                println!("marked {changed} discoveries explored");
                changes += 1;
            }
            if args.clear_lockouts {
                let changed = save_game.clear_all_lockouts();
                println!("cleared {changed} lockout timers");
                changes += 1;
            }
            if args.clear_mementos {
                let (weapons, items) = save_game.clear_mementos();
                println!("cleared mementos (weapons {weapons}, items {items})");
                changes += 1;
            }
            if args.complete_all_missions || args.reset_all_missions {
                let status = if args.complete_all_missions {
                    "Complete".to_string()
                } else {
                    "NotStarted".to_string()
                };
                let Some(value) = parse_status(&status) else {
                    eprintln!("error: bad status {status}");
                    return ExitCode::FAILURE;
                };
                let mut changed = 0usize;
                for (playthrough, list) in save_game
                    .mission_playthroughs()
                    .iter()
                    .map(|pt| pt.missions.len())
                    .enumerate()
                {
                    for mission in 0..list {
                        if save_game
                            .set_mission_status(playthrough, mission, value)
                            .is_ok()
                        {
                            changed += 1;
                        }
                    }
                }
                println!("{status} applied to {changed} missions");
                changes += 1;
            }
            if args.unlock_all_travel {
                let stations = load_stations_file(&args);
                match stations {
                    Some(stations) => {
                        let mut added = 0usize;
                        for entry in stations.all() {
                            if save_game.add_visited_teleporter(&entry.resource_name) {
                                added += 1;
                            }
                        }
                        println!("travel: added {added} stations");
                        changes += 1;
                    }
                    None => {
                        eprintln!(
                            "error: --unlock-all-travel needs --stations <Travel Stations.json>"
                        );
                        return ExitCode::FAILURE;
                    }
                }
            }
            if let Some(stage) = args.stage {
                let region_text = args.market.clone().unwrap_or_default();
                let key = region_text.trim().to_string();
                let regions = save_game.region_game_stages();
                let index = regions.iter().position(|region| {
                    region.region.eq_ignore_ascii_case(&key)
                        || region.region.to_lowercase().contains(&key.to_lowercase())
                });
                match index {
                    Some(index) => {
                        save_game
                            .set_region_game_stage(index, stage)
                            .expect("stage");
                        println!("{} game stage = {stage}", regions[index].region);
                        changes += 1;
                    }
                    None => {
                        eprintln!("error: no region matching '{region_text}'");
                        return ExitCode::FAILURE;
                    }
                }
            }
            if let Some(level) = args.prestige {
                save_game.set_i32(42, level.max(0));
                println!("challenge prestiges = {level}");
                changes += 1;
            }
            if let Some(id_text) = args.challenge_stat.clone() {
                let Some(value) = args.value else {
                    eprintln!("error: --challenge-stat ID needs --value V");
                    return ExitCode::FAILURE;
                };
                let Ok(id) = id_text.parse::<u16>() else {
                    eprintln!("error: '{id_text}' is not a numeric challenge id");
                    return ExitCode::FAILURE;
                };
                match save_game.set_challenge_stat(id, |entry| entry.total_value = value as i32) {
                    Ok(()) => {
                        println!("challenge {id} total value = {value}");
                        changes += 1;
                    }
                    Err(e) => eprintln!("error: {e}"),
                }
            }
            if let Some(pool) = args.ammo {
                let index = pool.max(1) - 1; // `--ammo` takes the 1-based list number
                match save_game.mutate_resource(index, |r| {
                    if let Some(amount) = args.amount {
                        r.amount = amount;
                    }
                    if let Some(level) = args.sdu {
                        r.upgrade_level = level;
                    }
                }) {
                    Ok(()) => {
                        println!("ammo pool #{pool} updated");
                        changes += 1;
                    }
                    Err(e) => eprintln!("note: {e}"),
                }
            }
            if let Some(value) = args.plot_mission {
                save_game.set_plot_mission_number(value);
                println!("plot mission = {value}");
                changes += 1;
            }
            if let Some(value) = args.active_mission {
                save_game.set_active_mission_number(value);
                println!("active mission = {value}");
                changes += 1;
            }
            if let (Some(playthrough), Some(index), Some(status_name)) = (
                args.mission_playthrough,
                args.mission_index,
                args.mission_status.clone(),
            ) {
                let Some(status) = parse_status(&status_name) else {
                    eprintln!(
                        "error: unknown mission status '{status_name}' (expected one of {MISSION_STATUSES:?})"
                    );
                    return ExitCode::FAILURE;
                };
                if let Err(e) = save_game.set_mission_status(playthrough, index, status) {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
                println!(
                    "playthrough {playthrough} mission {index}: status = {} ({status})",
                    mission_status_name(status)
                );
                changes += 1;
            }

            if changes == 0 {
                eprintln!("error: no edits specified (see --level, --currency, ...)");
                return usage();
            }

            let bytes = match container::serialize(&save_game.to_bytes(), args.platform) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            let out = args.out.clone().unwrap_or_else(|| args.file.clone());
            if out == args.file && args.backup {
                let bak = format!("{}.bak", args.file);
                if let Err(e) = std::fs::copy(&args.file, &bak) {
                    eprintln!("error: cannot create backup {bak}: {e}");
                    return ExitCode::FAILURE;
                }
                println!("backup: {bak}");
            }
            if let Err(e) = std::fs::write(&out, &bytes) {
                eprintln!("error: cannot write {out}: {e}");
                return ExitCode::FAILURE;
            }

            // Re-read and verify the written file.
            let verified = std::fs::read(&out).and_then(|d| {
                container::unpack(&d, args.platform).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                })
            });
            match verified {
                Ok(check) => {
                    let level = check.save_game().ok().and_then(|s| s.exp_level());
                    println!("wrote {} bytes to {out} ({changes} change(s))", bytes.len());
                    println!(
                        "verify: crc32={} huffman={} proto={} level={level:?}",
                        if check.crc32_valid() {
                            "ok"
                        } else {
                            "MISMATCH"
                        },
                        if check.huffman_stable() {
                            "ok"
                        } else {
                            "unstable"
                        },
                        if check.proto_stable() {
                            "ok"
                        } else {
                            "unstable"
                        },
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: verification failed: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => usage(),
    }
}
