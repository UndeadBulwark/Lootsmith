//! Asset indices resolve to names and back — using the bundled asset library on
//! the synthetic save.

mod common;

use common::synthetic_file;
use lootsmith_core::items::{self, PackedSlot};
use lootsmith_core::{proto::Message, AssetGroup};

#[test]
fn asset_names_resolve_and_round_trip() {
    let manager = lootsmith_core::bundled::asset_library().expect("bundled asset library");

    let save = synthetic_file();
    let top = Message::parse(&save.proto).expect("protobuf");

    let mut resolved = 0usize;
    for field in [53u32, 54u32] {
        for entry in top.get_all(field) {
            let sub = entry.as_message().unwrap().unwrap();
            let Some(serial) = sub.get(1).and_then(|f| f.as_bytes()) else {
                continue;
            };
            let decoded = items::unpack(serial, &manager.configs).expect("decode");
            let set = decoded.asset_library_set_id;

            let refs: Vec<(AssetGroup, items::PackedAssetReference)> = match &decoded.slot {
                PackedSlot::Weapon(w) => vec![
                    (AssetGroup::WeaponTypes, w.weapon_type),
                    (AssetGroup::BalanceDefs, w.balance),
                    (AssetGroup::Manufacturers, w.manufacturer),
                    (AssetGroup::WeaponParts, w.body_part),
                    (AssetGroup::WeaponParts, w.grip_part),
                    (AssetGroup::WeaponParts, w.barrel_part),
                    (AssetGroup::WeaponParts, w.elemental_part),
                    (AssetGroup::WeaponParts, w.material_part),
                    (AssetGroup::WeaponParts, w.prefix_part),
                    (AssetGroup::WeaponParts, w.title_part),
                ],
                PackedSlot::Item(it) => vec![
                    (AssetGroup::ItemTypes, it.item),
                    (AssetGroup::BalanceDefs, it.balance),
                    (AssetGroup::Manufacturers, it.manufacturer),
                    (AssetGroup::ItemParts, it.alpha_part),
                    (AssetGroup::ItemParts, it.material_part),
                ],
            };

            for (group, r) in refs {
                let name = manager
                    .lookup_asset(group, set, r.asset_index, r.sublibrary_index, r.use_set_id)
                    .unwrap_or_else(|e| panic!("lookup {r:?}: {e}"));
                if !r.is_none() {
                    assert!(name != "None");
                }
                // Name -> indices must reproduce the reference.
                let (ai, si, u) = manager
                    .find_asset(group, set, &name)
                    .unwrap_or_else(|e| panic!("find {name}: {e}"));
                assert_eq!(
                    (ai, si, u),
                    (r.asset_index, r.sublibrary_index, r.use_set_id),
                    "index round trip failed for {name}"
                );
                resolved += 1;
            }
        }
    }
    assert!(resolved > 0);
    eprintln!("OK: {resolved} asset references resolved and round-tripped to indices");
}
