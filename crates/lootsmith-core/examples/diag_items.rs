use lootsmith_core::gameinfo::AssetLibraryConfigs;
use lootsmith_core::items::{self, BlobStyle, PackedSlot};
use lootsmith_core::{container, proto::Message, Platform};

fn main() {
    let path = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    let data = std::fs::read(&path).expect("read fixture");
    let save = container::unpack(&data, Platform::PC).expect("unpack");
    let configs = AssetLibraryConfigs::from_path(std::path::Path::new(
        "/home/undeadbulwark/Projects/bl2-rogue/data/gibbed/Asset Library Manager.json",
    ))
    .expect("load configs");

    let top = Message::parse(&save.proto).expect("parse protobuf");
    for (field, label) in [(53u32, "item"), (54u32, "weapon")] {
        let mut idx = 0;
        for entry in top.get_all(field) {
            let sub = entry.as_message().unwrap().unwrap();
            let Some(serial) = sub.get(1).and_then(|f| f.as_bytes()) else {
                continue;
            };
            let decoded = match items::unpack(serial, &configs) {
                Ok(d) => d,
                Err(e) => {
                    println!("{label}[{idx}] decode error: {e}");
                    idx += 1;
                    continue;
                }
            };
            let reference = items::pack_styled(&decoded, &configs, BlobStyle::Reference).unwrap();
            let gamezero = items::pack_styled(&decoded, &configs, BlobStyle::GameZero).unwrap();
            let kind = match &decoded.slot {
                PackedSlot::Weapon(_) => "weapon",
                PackedSlot::Item(_) => "item",
            };
            println!(
                "{label}[{idx}] {kind} len={} id={} set={} | ref:{} ({}) game:{} ({})",
                serial.len(),
                decoded.unique_id,
                decoded.asset_library_set_id,
                reference.len(),
                if reference == serial { "MATCH" } else { "no" },
                gamezero.len(),
                if gamezero == serial { "MATCH" } else { "no" },
            );
            if let PackedSlot::Weapon(w) = &decoded.slot {
                println!(
                    "        weapon_type={:?} balance={:?} mfr={:?} grade={} stage={}",
                    w.weapon_type,
                    w.balance,
                    w.manufacturer,
                    w.manufacturer_grade_index,
                    w.game_stage
                );
                println!(
                    "        body={:?} grip={:?} barrel={:?} sight={:?} stock={:?}",
                    w.body_part, w.grip_part, w.barrel_part, w.sight_part, w.stock_part
                );
                println!(
                    "        elemental={:?} acc1={:?} acc2={:?} mat={:?} prefix={:?} title={:?}",
                    w.elemental_part,
                    w.accessory1_part,
                    w.accessory2_part,
                    w.material_part,
                    w.prefix_part,
                    w.title_part
                );
            }
            idx += 1;
        }
    }
}
