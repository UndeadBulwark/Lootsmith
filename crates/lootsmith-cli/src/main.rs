//! Lootsmith CLI: `inspect`, `verify`, and `dump`.

use lootsmith_core::{
    container, items, proto::Message, AssetGroup, AssetLibraryConfigs, AssetLibraryManager,
    Platform,
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
         lootsmith items   <file.sav> [--platform <id>]\n  \
         lootsmith edit    <file.sav> [--level N] [--currency N] [--golden-keys N] [--out OUT]\n\
         \n\
         Platforms: pc (default), x360, ps3, psvita, shield, switch\n\
         --dumps <Asset Library Manager.json> resolves asset names (or set LOOTSMITH_ASSET_LIBRARY).\n\
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
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--platform" | "-p" => args.platform = Platform::from_id(&it.next()?)?,
            "--dumps" | "-d" => args.dumps = Some(it.next()?),
            "--out" | "-o" => args.out = Some(it.next()?),
            "--no-backup" => args.backup = false,
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
            _ => return None,
        }
    }
    Some(args)
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

            let top = match Message::parse(&save.proto) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };
            if manager.is_none() {
                eprintln!(
                    "(no dumps file: showing raw indices; pass --dumps <Asset Library Manager.json>)"
                );
            }

            let mut count = 0usize;
            for (field, kind) in [(53u32, "item"), (54u32, "weapon")] {
                for entry in top.get_all(field) {
                    let sub = match entry.as_message() {
                        Ok(Some(m)) => m,
                        _ => continue,
                    };
                    let Some(serial) = sub.get(1).and_then(|f| f.as_bytes()) else {
                        continue;
                    };
                    let quantity = sub.get(2).and_then(|f| f.as_i32());
                    let equipped = sub.get(3).and_then(|f| f.as_bool());
                    count += 1;
                    match items::unpack(serial, &configs) {
                        Ok(decoded) => {
                            let set = decoded.asset_library_set_id;
                            match &decoded.slot {
                                items::PackedSlot::Weapon(w) => {
                                    println!(
                                        "{kind:>6} #{count} id={} balance={} mfr={} grade={} stage={}",
                                        decoded.unique_id,
                                        resolve(AssetGroup::BalanceDefs, set, w.balance),
                                        resolve(AssetGroup::Manufacturers, set, w.manufacturer),
                                        w.manufacturer_grade_index,
                                        w.game_stage
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
                                        "{kind:>6} #{count} id={} qty={:?} equipped={:?} \
                                         balance={} mfr={} grade={} stage={}",
                                        decoded.unique_id,
                                        quantity,
                                        equipped,
                                        resolve(AssetGroup::BalanceDefs, set, it.balance),
                                        resolve(AssetGroup::Manufacturers, set, it.manufacturer),
                                        it.manufacturer_grade_index,
                                        it.game_stage
                                    );
                                }
                            }
                        }
                        Err(e) => println!("{kind:>6} #{count} decode error: {e}"),
                    }
                }
            }
            println!("total: {count} packed items/weapons");
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
