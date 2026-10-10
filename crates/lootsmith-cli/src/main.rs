//! Lootsmith CLI: `inspect`, `verify`, and `dump`.

use lootsmith_core::{container, items, proto::Message, AssetLibraryConfigs, Platform};
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "lootsmith — a Borderlands 2 save editor (Rust port of Gibbed's tool)\n\
         \n\
         USAGE:\n  \
         lootsmith inspect <file.sav> [--platform <id>]\n  \
         lootsmith verify  <file.sav> [--platform <id>]\n  \
         lootsmith dump    <file.sav> [--platform <id>]\n  \
         lootsmith items   <file.sav> [--platform <id>]\n\
         \n\
         Platforms: pc (default), x360, ps3, psvita, shield, switch\n\
         Set LOOTSMITH_ASSET_LIBRARY to an 'Asset Library Manager.json' to override the built-in config."
    );
    ExitCode::from(2)
}

struct Args {
    command: String,
    file: String,
    platform: Platform,
}

fn parse_args() -> Option<Args> {
    let mut it = std::env::args().skip(1);
    let command = it.next()?;
    let file = it.next()?;
    let mut platform = Platform::PC;
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--platform" | "-p" => {
                let id = it.next()?;
                platform = Platform::from_id(&id)?;
            }
            _ => return None,
        }
    }
    Some(Args {
        command,
        file,
        platform,
    })
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
            let configs = match std::env::var("LOOTSMITH_ASSET_LIBRARY") {
                Ok(p) => match AssetLibraryConfigs::from_path(std::path::Path::new(&p)) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: cannot load {p}: {e}");
                        return ExitCode::FAILURE;
                    }
                },
                Err(_) => AssetLibraryConfigs::pc_default(),
            };
            let top = match Message::parse(&save.proto) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };
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
                        Ok(decoded) => match &decoded.slot {
                            items::PackedSlot::Weapon(w) => println!(
                                "{kind:>6} #{count} len={} id={} set={} type={:?} balance={:?} \
                                 mfr={:?} grade={} stage={}",
                                serial.len(),
                                decoded.unique_id,
                                decoded.asset_library_set_id,
                                w.weapon_type,
                                w.balance,
                                w.manufacturer,
                                w.manufacturer_grade_index,
                                w.game_stage
                            ),
                            items::PackedSlot::Item(it) => println!(
                                "{kind:>6} #{count} len={} id={} set={} qty={:?} equipped={:?} \
                                 balance={:?} mfr={:?} grade={} stage={}",
                                serial.len(),
                                decoded.unique_id,
                                decoded.asset_library_set_id,
                                quantity,
                                equipped,
                                it.balance,
                                it.manufacturer,
                                it.manufacturer_grade_index,
                                it.game_stage
                            ),
                        },
                        Err(e) => println!("{kind:>6} #{count} decode error: {e}"),
                    }
                }
            }
            println!("total: {count} packed items/weapons");
            ExitCode::SUCCESS
        }
        _ => usage(),
    }
}
