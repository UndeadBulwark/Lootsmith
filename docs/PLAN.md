# Lootsmith — Plan, Status & Handoff

A cross-platform Borderlands 2 save editor written in Rust.

**Repo:** `~/Projects/lootsmith` · **HEAD:** `e692c58` (working tree clean)
**Updated:** 2026-10-10 · **Reference:** Gibbed's Borderlands 2 Save Editor (C#, zlib) · upstream `github.com/gibbed/Gibbed.Borderlands2`

> This file is the single source of truth after a context compaction. Read §1 (checklist),
> §3 (commands) and §4 (facts/gotchas) first.

---

## 1. Status checklist

### Core library (`lootsmith-core`) — done & validated
- [x] Container: X360-CON reject, SHA1, `u32BE` length, LZO/zlib (single block + block table), WSG header, CRC32
- [x] Bit primitives (`bitio`: LSB-first reader/writer)
- [x] Huffman codec (`huffman`: MSB-first, pre-order tree)
- [x] Raw-preserving protobuf codec (`proto`) — decode→encode byte-identical, unknown fields kept
- [x] Typed top-level `WillowTwoPlayerSaveGame` (`savegame`, fields 1–58) + accessors + dump
- [x] Game data (`gameinfo`): Asset Library Manager parse (version + per-group bit widths), name lookup / find / choices
- [x] Item codec (`items`): `PackedAssetReference`, `PackedItem`, `PackedWeapon`, obfuscation, checksum, `unpack`/`pack`, `set_part`/`part`
- [x] Write path: `container::wrap` / `serialize` (LZO via `lzokay`, zlib via `flate2`)

### CLI (`lootsmith-cli`, binary `lootsmith`) — done
- [x] `inspect`, `verify`, `dump`, `items` (`--container bag|bank`), `skills`, `travel` (`--stations`), `vehicles` (`--customizations`), `missions` (`--missions-json`)
- [x] `edit`: `--level/--exp-points/--general-skill-points/--specialist-skill-points/--playthroughs/--currency/--golden-keys/--bank-slots`, `--item N --part SLOT --set NAME [--container bag|bank]`, `--skill N [--grade G] [--points P]`, `--travel-add/--travel-remove/--travel-last NAME`, `--steering-mode N`, `--plot-mission N`/`--active-mission N`, `--mission-playthrough P --mission-index I --mission-status NAME`, `--out`, `.bak` backup, re-verify

### GUI (`lootsmith-gui`) — full editor, 10 tabs + shell
- [x] Toolbar (Open/Save/Save As/Reload/Game data) + Ctrl+O/S/Shift+S/R; themed shell; light/dark toggle
- [x] Persisted settings (save, game-data folder, theme, tab) via eframe storage
- [x] Load a game-data *folder*: Asset Library + Travel Stations + Customizations + Missions auto-discovered
- [x] Dirty tracking / unsaved indicator
- [x] Tabs: **General**, **Character**, **Currency**, **Skills**, **Fast Travel**, **Vehicles**, **Missions**, **Backpack**, **Bank**, **Raw**
- [x] Character: name, level/exp/skill points, Head & Skin pickers (resolved by class usage)
- [x] Raw: every top-level field; edit scalars directly
- [x] Item system: new weapon/item, duplicate, **`BL2(...)` item codes (copy/paste)**, level sync (all/equipped), quantity/equipped/mark/quick-slot, part editor, move bank↔backpack, remove
- [x] Fast Travel: resolved station list, add/remove/set-last
- [x] Skills: skill-tree calculator (embedded trees/tiers/icons for all 6 classes; click to add/remove; tier gating; spent/available)
- [x] Vehicles: per-family Skin 1/Skin 2 + steering mode
- [x] Missions: plot/active numbers, per-mission status, challenge list
- [x] Save discovery: `locate` module + `lootsmith saves` + GUI "Find saves" picker; first run auto-opens newest (native Aspyr, Steam/Proton appid 49520, Flatpak, Wine/Lutris/custom prefixes)
- [x] `.desktop` launcher + icon (bundled window icon + hicolor app icon)

### Remaining vs Gibbed (smaller gaps)
- [ ] **New save** (blank / from player class) — Gibbed's New + New-from-class toolbar
- [ ] **Import sections from another save** (skills / missions / world / stats)
- [ ] **Overpower level** fields (59–61) — serialization needs investigation
- [ ] Raw: edit list/message fields (currently scalars only)
- [ ] Character colors (UIPreferences fields 2–4)
- [ ] Balance-constrained item part pickers (searchable) — nicety
- [ ] `convert` (platform↔platform); JSON import/export
- [ ] Reserve crate names on crates.io; package Windows/macOS
- [ ] **In-game verification of an edited save (human step)**

### Environment / tooling (already done)
- [x] Rust 1.98.1; installed `xdotool wmctrl xorg-xwd` (optional for the live window)
- [x] KDE entry: `~/.local/share/applications/lootsmith.desktop`

---

## 2. Repo map

```
~/Projects/lootsmith
├─ Cargo.toml                      # workspace; version 0.0.1, edition 2021, license Zlib
├─ README.md, LICENSE (zlib), .github/workflows/ci.yml (fmt+clippy+test)
├─ scripts/lootsmith-gui           # wrapper: builds release on first run, execs it
├─ scripts/gui-drive.sh            # OPTIONAL: launch on X11 + screenshot (human use)
├─ tests/fixtures/pc_Save0001.sav  # gitignored real PC save (see §3)
└─ crates/
   ├─ lootsmith-core/src/  lib.rs error.rs crc32.rs bitio.rs huffman.rs proto.rs
   │                       platform.rs container.rs savegame.rs gameinfo.rs items.rs
   ├─ lootsmith-core/tests/ spike.rs proto_roundtrip.rs items.rs names.rs edit.rs
   ├─ lootsmith-core/examples/ diag.rs diag_items.rs     # dev diagnostics
   ├─ lootsmith-cli/src/main.rs                          # binary name: lootsmith
   ├─ lootsmith-gui/src/   lib.rs app.rs main.rs         # lib lootsmith_gui + bin lootsmith-gui
   ├─ lootsmith-gui/tests/ui.rs                          # egui_kittest UI tests
   ├─ lootsmith-gui/examples/snapshot.rs                 # offscreen PNG renderer
   └─ lootsmith/src/lib.rs                               # facade re-exporting core
```

Dependencies: `lzo` (decompress), `lzokay` (compress), `flate2`, `sha1`, `serde`+`serde_json`,
`eframe`/`egui` 0.36, `rfd` 0.17. Dev: `egui_kittest` 0.36 (features wgpu, snapshot), `image`.

---

## 3. How to build / test / run / automate

```sh
cd ~/Projects/lootsmith

# Build / lint / test
cargo build --release
cargo clippy --workspace --all-targets -- -D warnings   # must stay clean
cargo test --workspace                                   # 21 tests

# CLI  (binary is `lootsmith`)
cargo run -p lootsmith-cli -- inspect <save>
cargo run -p lootsmith-cli -- verify  <save>
cargo run -p lootsmith-cli -- dump    <save>
cargo run -p lootsmith-cli -- items   <save> --dumps "$DUMPS" [--container bag|bank]
cargo run -p lootsmith-cli -- skills  <save>
cargo run -p lootsmith-cli -- travel  <save> --stations "$STATIONS"
cargo run -p lootsmith-cli -- vehicles <save> --customizations "$CUSTOMS"
cargo run -p lootsmith-cli -- missions <save> --missions-json "$MISSIONS"
cargo run -p lootsmith-cli -- edit    <save> --level 61 --currency 1234567 --out /tmp/e.sav
cargo run -p lootsmith-cli -- edit    <save> --item 6 --part elemental \
      --set "GD_Weap_Pistol.elemental.Pistol_Elemental_Fire" --dumps "$DUMPS" --out /tmp/e.sav

# GUI
cargo run -p lootsmith-gui                       # native window
cargo run -p lootsmith-gui -- --open <save> --dumps "$DUMPS"
cargo run -p lootsmith-gui -- --selftest --open <save>          # headless load/edit/save/reload

# GUI: headless widget-level tests (by label, no window/focus)
cargo test -p lootsmith-gui

# GUI: offscreen screen renders to PNG (no window/focus)
cargo run -p lootsmith-gui --example snapshot -- /tmp/opencode/shots
```

**Paths used throughout:**
- Real save: `~/bl2-zluda/prefix-baseline/drive_c/users/steamuser/Documents/My games/Borderlands 2/WillowGame/SaveData/76561198098137537/Save0001.sav`
- Test fixture (copy): `tests/fixtures/pc_Save0001.sav` (9,548 B; sha256 `475b2581507ac799cb71e026675e1c868f5f3bfe7c22ea917b61536708a4d7bd`)
- Dumps: `~/Projects/bl2-rogue/data/gibbed/Asset Library Manager.json` (the reusable dumps live here: Items, Weapon/Item Parts, Balance lists, Missions, Travel Stations, …)
- GUI log: `~/.cache/lootsmith/gui.log`
- Launcher: `~/.local/share/applications/lootsmith.desktop` → `scripts/lootsmith-gui %f`; icon `~/.local/share/icons/hicolor/256x256/apps/lootsmith.png`

---

## 4. Key facts & gotchas (hard-won)

**Save container has TWO compression layers**
`File = SHA1(20) || u32BE(inner_len) || compress(Inner)`.
`Inner = u32BE(inner_size=len-4) || "WSG" || u32 version=2 || u32 crc32(proto) || i32 proto_size || Huffman(proto)`
(platform endianness for the inner fields; the outer length/block table are big-endian). Switch (`None`) writes `Inner` raw.

**Three byte-identity corrections (do not re-litigate):**
1. **Huffman is not byte-reproducible.** The game's Huffman tree differs from Gibbed's (and ours); both decode to the same protobuf. Gate = *protobuf byte-identity* + `huffman_stable()` (`decode(encode(proto)) == proto`), **not** raw inner bytes.
2. **Outer compressed bytes are not byte-reproducible** (MiniLZO vs `lzokay`; SharpZipLib vs `flate2`). Gate on the unwrapped inner buffer, not the whole file.
3. **Item blobs:** real-content blobs (non-zero unique id) *are* byte-identical using the reference convention; the blank 40-byte placeholders (id 0, set 255) are zero-filled by the game and are only semantically stable (`BlobStyle` exposes both).

**Protobuf:** raw-preserving codec — each field keeps exact bytes, so untouched fields re-emit exactly and unknown fields survive; setters replace one field canonically. Typed top-level is fields 1–58 (59–61 have no `[ProtoMember]` and are ignored).

**Asset library:** version 7; per-group bit widths (ItemTypes 9/8, ItemParts 6/10, Manufacturers 4/7, BalanceDefs 10/10, WeaponTypes 7/6, WeaponParts 6/11) exposed as `AssetLibraryConfigs::pc_default()` (matches the dumps). Only X360 has sublibrary remapping; PC lookups are direct. Item blob header: 7-bit version, 1-bit is-weapon, 32-bit unique id, `0xFFFF` marker, 8-bit set id.

**Block table:** we write `ceil(len / 0x40000)` blocks (deliberately fixing the reference's off-by-one for exact multiples).

**CRC32** = IEEE; **SHA1** covers the wrapper *including* the `u32BE` length.

**Platform matrix:** PC=LZO/Little · X360=LZO/Big · PS3=zlib/Big · PSVita=zlib/Little · Shield=zlib/Little · Switch=None/Little.

**GUI/egui 0.36 API specifics** (differs from older egui): `eframe::App` uses `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame)`; panels are `egui::Panel::top(id)/::bottom(id)` shown into the root `Ui`; `CentralPanel::default().show(ui, …)`; `ComboBox::from_id_salt`.

**Automation:** `egui_kittest` manipulates by widget **label** via AccessKit (no window, no focus); `harness.render()` renders offscreen. XTest coordinate clicks do **not** work under KWin/Wayland — don't go back to that.

**Game data is not bundled.** The library loads the Asset Library Manager JSON at runtime (dumps dir). Keep it that way (licensing).

---

## 5. Remaining work (next slices)

Each slice = core typing (if needed) + a GUI tab + a `egui_kittest` interaction test + an offscreen snapshot.

Done: **Backpack / Bank** — `Container {BackpackItem, BackpackWeapon, Bank}` in
`savegame.rs`; `entries_in`/`bank_entries`/`entries`, `append_entry`/`remove_entry`/
`move_entry`/`mutate_entry` and quantity/equipped setters; Backpack/Bank GUI tabs.
Done: **Skills** — `Skill {skill, grade, grade_points, equipped_slot_index}` (field 8);
`skills()` + `set_skill_grade`/`set_skill_grade_points`; `skills` CLI + Skills tab.

Done: **Fast Travel** — `visited_teleporters`/`last_visited_teleporter` + add/remove/set
(fields 16/17); `travel` module parsing `Travel Stations.json`; travel CLI + tab.
Done: **Vehicles** — `VehicleCustomization` {family, customizations}; `vehicles()`,
`vehicle_steering_mode` + setter, `set_vehicles`; `VEHICLE_FAMILIES`; `customizations`
module parsing `Customizations.json`; vehicles CLI + tab.
Done: **Missions & challenges** — `Challenge`/`MissionEntry`/`MissionPlaythrough`;
`challenges()`, `mission_playthroughs()`, plot/active mission getters+setters,
`set_mission_status`; `missions` module parsing `Missions.json`; missions CLI + tab.

Polished parity is complete; the remaining work is polish only:

1. **Item choices by balance** — constrain the item-editor dropdowns using the
   Weapon/Item Balance Part Lists (nicety; the full lists already work).
2. **`convert`** platform↔platform; JSON import/export.
3. **Packaging** — reserve crate names; Windows/macOS bundles.
4. **In-game verification** (human step).

Also: item editor "choices constrained by balance part lists" (Weapon/Item Balance Part Lists.json); `convert` (platform↔platform); JSON export/import; reserve crate names (`lootsmith`, `-core`, `-cli`, `-gui`); Windows/macOS packaging; **in-game verification** (human).

Note: earlier the C# sub-message sources were fetched to `/tmp/items` and `/tmp/pbf` — those are ephemeral. Re-fetch from `raw.githubusercontent.com/gibbed/Gibbed.Borderlands2/master/projects/Gibbed.Borderlands2.ProtoBufFormats/WillowTwoSave/<Name>.cs` when typing sub-messages.

---

## 6. Original phased plan (condensed)

| Phase | Scope | State |
|---|---|---|
| 0 | Setup, CI, license, spike | ✅ |
| 1 | Container (SHA1/LZO/zlib/blocks) | ✅ |
| 2a | Bit + Huffman codecs | ✅ |
| 2b | Protobuf codec (byte-identical) | ✅ |
| 3 | Game data / name resolution | ✅ |
| 4 | Item codec | ✅ |
| 5 | CLI edit + write path | ✅ |
| 6 | GUI | ✅ 10 tabs (General/Character/Currency/Skills/Fast Travel/Vehicles/Missions/Backpack/Bank/Raw) + toolbar, item codes, theme |

Naming/licensing: zlib, inherited from upstream; README attribution required; keep `Copyright (c) 2019 Rick Gibbed`; "Borderlands" descriptive only; do not bundle game data.

## 7. History (git)

`51a6093` container/Huffman spike · `5d0d98c` protobuf · `8f9e2b0` name resolution ·
`1211931` item codec · `da30bfd` write path + edit · `869432f` item editing by name ·
`780d2d7` GUI scaffold · `eba0b57` GUI slice 1 · `668513f` headless UI testing + snapshot tool ·
`358e873` backpack/bank · `41e16da` skills · `36a316f` fast travel · `41ebe45` vehicles ·
`9f4ff00` missions & challenges · `58e8429` GUI Phase 0 (shell/theme/config/dirty) ·
`3965b3d` GUI Phase 2 (item codes/creation/sync) · `2fe7486` GUI Phase 3 (Character + Raw) ·
`84a1f2c` save auto-discovery (locate + Find saves) · `ebcce76` final icon · `36aaa13` docs ·
`b0cd3fe` GUI redesign (theme/layout/table/cards) · `e692c58` skill-tree calculator · (HEAD)
