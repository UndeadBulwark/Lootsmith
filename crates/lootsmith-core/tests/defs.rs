//! Real-dump checks for the part-picker definitions. These use the bundled
//! community tables, so they run with no game-data folder.

const PISTOL_TYPE: &str = "GD_Weap_Pistol.A_Weapons.WeaponType_Jakobs_Pistol";
const PISTOL_BALANCE: &str = "GD_Weap_Pistol.A_Weapons.Pistol_Jakobs_4_VeryRare";

#[test]
fn jakobs_pistol_inherits_parts_from_its_base_chain() {
    let defs = lootsmith_core::bundled::defs().expect("bundled defs parse");

    assert!(
        defs.weapon_is_suitable(PISTOL_BALANCE, PISTOL_TYPE),
        "the leaf balance must be suitable for its weapon type"
    );

    // `body` comes from the leaf; `grip`/`barrel` are inherited from the
    // `..._3_Rare` / root balances (the leaf lists neither).
    let body = defs
        .weapon_slot_choices(PISTOL_TYPE, PISTOL_BALANCE, "body")
        .expect("body is a part slot");
    assert!(
        body.iter().any(|p| p.ends_with("Pistol_Body_Jakobs_4")),
        "leaf body part expected, got {body:?}"
    );

    let grip = defs
        .weapon_slot_choices(PISTOL_TYPE, PISTOL_BALANCE, "grip")
        .expect("grip is a part slot");
    assert!(
        grip.iter().any(|p| p.ends_with("Pistol_Grip_Bandit")),
        "grip part inherited through the base chain expected, got {grip:?}"
    );
    assert!(
        grip.len() < 20,
        "the slot list must be the resolved set, not the whole library: {grip:?}"
    );

    // `accessory2` is a real slot even though this balance lists no parts for
    // it (Gibbed always shows the combobox).
    assert_eq!(
        defs.weapon_slot_choices(PISTOL_TYPE, PISTOL_BALANCE, "accessory2"),
        Some(vec![]),
        "accessory2 must exist as a slot"
    );
}

#[test]
fn wrong_weapon_type_is_rejected() {
    let defs = lootsmith_core::bundled::defs().expect("bundled defs parse");
    assert!(!defs.weapon_is_suitable(
        PISTOL_BALANCE,
        "GD_Weap_SniperRifles.A_Weapons.WeaponType_Jakobs_Sniper"
    ));
}
