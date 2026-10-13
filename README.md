# Lootsmith

A Linux-native, Rust ([egui](https://github.com/emilk/egui)) save editor for
**Borderlands 2** — a port/replacement for Gibbed's Borderlands 2 Save Editor.

[![CI](https://github.com/UndeadBulwark/lootsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/UndeadBulwark/lootsmith/actions/workflows/ci.yml)
[![License: Zlib](https://img.shields.io/badge/license-Zlib-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)

**Current state — `0.4.0`.** The core library and CLI are complete and validated
against real PC saves; the GUI is a full editor with a themed, responsive shell.
Status and the roadmap to 1.0 live in [`docs/PLAN.md`](docs/PLAN.md).

<img src="docs/screenshots/lootsmith.png" width="180" alt="Lootsmith">

<sub>Lootsmith runs in a Light theme (default) and a Dark theme — see the
[comparison below](#screenshots).</sub>

Lootsmith reads, edits, and writes Borderlands 2 save files: items, weapons,
currency, skills, and progression. It ships as a library (`lootsmith-core`), a CLI
(`lootsmith-cli`), and an `egui` desktop app (`lootsmith-gui`), with `lootsmith` as
a facade crate.

> Lootsmith is a Rust port based on Gibbed's Borderlands 2 Save Editor by Rick
> Gibbed (zlib license).

## Features

- **Save format:** LZO/zlib container (SHA-1, CRC-32, Huffman), all 58 top-level
  fields of `WillowTwoPlayerSaveGame`, byte-exact round trips.
- **Items & weapons:** part editing with balance-constrained pickers, display
  names, rarity, and `BL2(…)` item codes.
- **Skills:** every vault hunter's tree matches the in-game layout, with the real
  per-skill columns, the action-skill banner, and tier gating.
- **Everything else:** General, Character, Currency, Ammo & SDUs, World, Fast
  Travel, Vehicles, Missions, Backpack, Bank, Raw, About, plus `profile.bin`
  (golden keys, badass rank/tokens, bonuses).
- **Save discovery:** native, Steam/Proton, and Wine prefixes.
- **Platform conversion:** PC, Xbox 360, PS3, PS Vita, Shield, Switch.
- **Game data:** bundled community tables (with a user-folder override) so names
  resolve on a fresh run with no setup.

## Screenshots

Light theme (the default) on top, dark below — the same Backpack workbench:

[![Light and dark themes](docs/screenshots/theme-compare.png)](docs/screenshots/theme-compare.png)

More (click for full size): [Skills, light](docs/screenshots/skills-light.png) ·
[Skills, dark](docs/screenshots/skills-dark.png) ·
[collapsed rail, light](docs/screenshots/sidebar-collapsed-light.png).

## Install / build

```sh
git clone https://github.com/UndeadBulwark/lootsmith
cd lootsmith
cargo build --release      # binaries in target/release/
```

Rust 1.85+ is required. The GUI builds on `eframe`/`wgpu`.

## Run

```sh
# GUI
cargo run -p lootsmith-gui

# CLI — inspect a save
cargo run -p lootsmith-cli -- inspect <save.sav>
```

### CLI at a glance

```
inspect  verify  dump     items   skills  travel  vehicles  missions  saves
new  json  unjson  profile  ammo  sdus  challenges  world  edit
```

`edit` covers level/points, currency, golden keys, bank slots, `--item/--part/--set`,
`--skill`, travel, `--steering-mode`, missions, `--ammo/--amount/--sdu`,
`--market … --sdu-level L`, `--backpack-slots`, `--bank-size`, `--max-sdus`, with
`--out`, `.bak` backups, and re-verify.

## Workspace layout

| Crate | Purpose |
|---|---|
| `lootsmith-core` | Container, bit/Huffman codecs, protobuf codec, item codec, game data |
| `lootsmith-cli`  | Thin command layer over core (binary `lootsmith`) |
| `lootsmith-gui`  | egui desktop app |
| `lootsmith`      | Facade crate |

Bundled game-data tables live under `crates/lootsmith-core/assets/data/`; their
sources, revisions, and licences are recorded in
[`PROVENANCE.md`](crates/lootsmith-core/assets/data/PROVENANCE.md). A folder you
pick always overrides the bundled copy.

## Development

```sh
cargo build --release
cargo clippy --workspace --all-targets -- -D warnings   # must stay clean
LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace
cargo run -p lootsmith-gui --example snapshot -- /tmp/shots
```

The GUI test suite runs on a synthetic fixture (`lootsmith_core::fixture`), never a
personal save; snapshot baselines live in `crates/lootsmith-gui/tests/snapshots/`.
See [`AGENTS.md`](AGENTS.md) and [`docs/PLAN.md`](docs/PLAN.md) for the milestone
gate and current roadmap.

## Credits & licence

- Original tool: Gibbed's Borderlands 2 Save Editor by Rick Gibbed.
- Lootsmith is licensed under the [Zlib licence](LICENSE).

## AI disclaimer

Made with the assistance of AI and primarily for personal use. Use at your own
risk.
