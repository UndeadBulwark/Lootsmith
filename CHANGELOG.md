# Changelog

All notable changes to Lootsmith are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Desktop GUI (`lootsmith-gui`, egui): General / Character / Currency / Ammo &
  SDUs / World / Skills / Fast Travel / Vehicles / Missions / Backpack / Bank /
  Raw / About, save discovery, item codes, `profile.bin` editing, new/import.
- Themed shell (wood bars, sidebar, status bar) with Auto/1x–4x UI scaling.
- Bundled community game-data tables under `crates/lootsmith-core/assets/data/`
  with a user-folder override; derived `rarity.json`.
- CLI (`lootsmith-cli`): inspect / verify / dump / items / skills / travel /
  vehicles / missions / ammo / sdus / challenges / world / saves / new / json /
  unjson / profile / edit.
- Cross-platform container support (PC, X360, PS3, PSVita, Shield, Switch).

### Changed
- Skills: the calculator now uses the real per-skill columns from the source
  page (all six vault hunters), so every tree matches the in-game layout, and
  each class shows its action skill as a wide banner instead of a small tile.
  Adds pure board rules (`skill_rules`) that enforce the point budget and block
  a removal that would lock a higher tier, a details pane, Respec, and an
  `LS2:` build string copy/paste. The tree panels are laid out on fixed rects so
  the three columns never clip.
- Sidebar is collapsible to a 64 px icon rail (toggle + per-tab drawn icons);
  the state is remembered between runs.
- Changing the class refunds every spent skill point and clears the old class's
  ranks, so the header and the new class's tree stay consistent.

### Fixed
- Item/weapon part editor now matches Gibbed: every slot is shown (including
  Accessory 2, Stock and Elemental), and each slot offers only the parts the
  balance's `base` chain resolves to, instead of the whole asset library.
- Item display names no longer blank out when a game-data folder without the
  Gibbed `Name Parts` files is selected — the bundled tables are used as a
  fallback. Non-weapon items also fall back to their `Items.json` name.
- Currency tab: slot order corrected (Torgue tokens are index 4, not 3) and the
  real currencies stay visible; reserved slots are hidden unless "show unused".
- Removed the misleading "Golden keys" field from the General tab — field 48 is
  `NumGoldenKeysNotified`, not the key balance (use the Profile tab).
- Part pickers now show human labels (prefix/title words, item flavour names,
  otherwise the last path segment); an out-of-balance part warns in the CLI and
  GUI instead of being silently allowed.
- Fast Travel lists visited stations the station table does not know under an
  "Unknown" group instead of hiding them.

## [0.0.1] - 2026-10-10
- Initial public snapshot: core container/codec/protobuf/item work, CLI, and the
  GUI.

[Unreleased]: https://github.com/UndeadBulwark/lootsmith/commits/main
