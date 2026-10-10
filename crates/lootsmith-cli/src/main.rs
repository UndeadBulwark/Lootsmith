//! Lootsmith CLI: `inspect`, `verify`, and `dump`.

use lootsmith_core::{
    container, items, mission_status_name, AssetGroup, AssetLibraryConfigs, AssetLibraryManager,
    Container, Customizations, Missions, Platform, TravelStations, MISSION_STATUSES,
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
         lootsmith items   <file.sav> [--platform <id>] [--container bag|bank]\n  \
         lootsmith skills  <file.sav> [--platform <id>]\n  \
         lootsmith travel  <file.sav> [--stations FILE]\n  \
         lootsmith vehicles <file.sav> [--customizations FILE]\n  \
         lootsmith missions <file.sav> [--missions-json FILE]\n  \
         lootsmith edit    <file.sav> [--level N] [--currency N] [--golden-keys N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --item N --part SLOT --set NAME --dumps FILE [--container bag|bank]\n  \
         lootsmith edit    <file.sav> --skill N [--grade G] [--points P] [--out OUT]\n  \
         lootsmith edit    <file.sav> [--travel-add NAME] [--travel-remove NAME] [--travel-last NAME] [--out OUT]\n  \
         lootsmith edit    <file.sav> [--steering-mode N] [--out OUT]\n  \
         lootsmith edit    <file.sav> [--plot-mission N] [--active-mission N] [--out OUT]\n  \
         lootsmith edit    <file.sav> --mission-playthrough P --mission-index I --mission-status NAME [--out OUT]\n\
         \n\
         Platforms: pc (default), x360, ps3, psvita, shield, switch\n\
         --dumps <Asset Library Manager.json> resolves asset names (or set LOOTSMITH_ASSET_LIBRARY).\n\
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
    let mut args = Args {
        command: it.next()?,
        file: it.next()?,
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
            _ => return None,
        }
    }
    Some(args)
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
                                println!(
                                    "{kind:>6} #{count} id={} balance={} mfr={} grade={} \
                                     stage={}{meta}",
                                    decoded.unique_id,
                                    resolve(AssetGroup::BalanceDefs, set, w.balance),
                                    resolve(AssetGroup::Manufacturers, set, w.manufacturer),
                                    w.manufacturer_grade_index,
                                    w.game_stage,
                                );
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
                                    resolve(AssetGroup::WeaponParts, set, w.prefix_part),
                                    resolve(AssetGroup::WeaponParts, set, w.title_part),
                                );
                            }
                            items::PackedSlot::Item(it) => {
                                println!(
                                    "{kind:>6} #{count} id={} balance={} mfr={} grade={} \
                                     stage={}{meta}",
                                    decoded.unique_id,
                                    resolve(AssetGroup::BalanceDefs, set, it.balance),
                                    resolve(AssetGroup::Manufacturers, set, it.manufacturer),
                                    it.manufacturer_grade_index,
                                    it.game_stage,
                                );
                            }
                        }
                    }
                    Err(e) => println!("{kind:>6} #{count} decode error: {e}"),
                }
            }
            println!("total: {count} packed blobs (backpack {backpack}, bank {bank})");
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
                .map(|path| (path.clone(), TravelStations::from_path(std::path::Path::new(&path))));
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
