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

## [0.0.1] - 2026-10-10
- Initial public snapshot: core container/codec/protobuf/item work, CLI, and the
  GUI.

[Unreleased]: https://github.com/UndeadBulwark/lootsmith/commits/main
