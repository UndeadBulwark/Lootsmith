# Lootsmith — Plan, Status & Handoff

A cross-platform Borderlands 2 save editor written in Rust.

**Repo:** `~/Projects/lootsmith` · **Reference:** Gibbed's Borderlands 2 Save Editor
(C#, zlib) · upstream `github.com/gibbed/Gibbed.Borderlands2` · agent brief: `AGENTS.md`.

> **This file is the single source of truth.** The `~/.opencode/plan/*.md` files are
> pointers to it — do not keep separate plans. Read §0 (the milestone gate), §1
> (status), §2 (roadmap), §6 (facts/gotchas) and §7 (commands) first.

---

## 0. How a milestone is done (the gate)

A milestone is **not done** until **all** of these hold:

1. `cargo build --release`; `LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace` —
   green, **no skips** (a skipped test counts as not done).
2. `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo fmt --all -- --check` clean.
3. Its **review bundle exists** — `scripts/review-bundle.sh` →
   `artifacts/bundles/lootsmith-review-<short-hash>.zip` (§5). The bundle is built from
   **current** code, never copied.
4. The report uses the **standard template** and **ends with the line**
   `Bundle: <full path>`.

Standard report template: what changed · test line (suites / passed / failed / skipped /
ignored, from a fresh `LOOTSMITH_REQUIRE_FIXTURES=1` run) · clippy + fmt result · the
contrast ratchet table (per screen, dark/light) · PNGs for every touched screen in
**dark and light** · known differences from the mockup (numbered) · open issues /
anything not verified · `Bundle: <path>`.

---

## 1. Status checklist

### Core library (`lootsmith-core`) — done & validated
- [x] Container: X360-CON reject, SHA1, `u32BE` length, LZO/zlib blocks, WSG header, CRC32
- [x] Bit primitives (`bitio`), Huffman codec (`huffman`), raw-preserving protobuf codec (`proto`)
- [x] Typed top-level `WillowTwoPlayerSaveGame` (`savegame`, fields 1–58) + accessors + dump
- [x] Game data (`gameinfo`): Asset Library Manager parse, name lookup / find / choices
- [x] Item codec (`items`): refs, obfuscation, checksum, `unpack`/`pack`, `set_part`/`part`,
      `game_stage()`/`manufacturer_grade()` (+ setters), `into_packed`
- [x] Write path: `container::wrap`/`serialize` (LZO via `lzokay`, zlib via `flate2`)
- [x] Section import (`import_fields_from`), new saves (`blank`, `make_fresh_character`, `refresh_guid`)
- [x] `savejson`: wire-faithful save ↔ JSON (byte-identical round trip; CLI `json`/`unjson`)
- [x] `profile`: profile.bin — golden keys triplet codec, badass rank/tokens, 14 bonus stats
      (base-32 codec), 1001-byte customization unlock list
- [x] `partlists`: Balance Part Lists → constrained pickers
- [x] `nameparts`: Weapon/Item Name Parts + Items.json → item display names
- [x] `rarity`: data-driven classification (`assets/rarity.json`); unmatched = `Unknown`, never guessed
- [x] `fixture`: `synthetic_save()` so CI runs without a personal save
- [x] Ammo pools (field 11), SDUs (36/56/13), challenges (15/38/39/42), world (29/30/43/46)

### CLI (`lootsmith-cli`, binary `lootsmith`) — done
- [x] `inspect`, `verify`, `dump`, `items`, `skills`, `travel`, `vehicles`, `missions`,
      `saves`, `ammo`, `sdus`, `challenges`, `world`, `new --class NAME`, `json`/`unjson`, `profile`
- [x] `edit`: level/points/currency/golden keys/bank slots · `--item/--part/--set` · `--skill` ·
      travel · `--steering-mode` · missions · `--ammo/--amount/--sdu` · `--market … --sdu-level L` ·
      `--backpack-slots S` / `--bank-size S` / `--max-sdus` · `--out`, `.bak`, re-verify

### GUI (`lootsmith-gui`) — full editor, themed shell
- [x] Homestead shell: plank top bar, grouped sidebar (counts, logo), status plank; Auto/1x–4x scale
- [x] `theme::Palette` + WCAG contrast gate; drawn pixel-art widget set (`ui/pixel.rs`, `ui/widgets.rs`)
- [x] Tabs: General, Character, Currency, Ammo & SDUs, World, Skills, Fast Travel, Vehicles,
      Missions, Backpack (n), Bank (n), Raw, About
- [x] `ui/item_table.rs` list + item workbench (parts union, editable stat blocks)
- [x] Save discovery, `BL2(…)` item codes, `profile.bin` editing, new/import, platform Save As
- [x] **M3 complete** — Backpack/Bank match the mockup; side-by-side review sheets done

### Remaining
- [ ] **M4** — every screen except Skills onto Homestead; contrast hard-zero (§3)
- [ ] **M5** — Skills redesign (drop-in ready; needs the column prerequisite) (§4)
- [ ] **M6** — consistency lock (enable `lint_tokens`, drive ratchet to 0, byte-exact theme switch)
- [ ] **M7** — QoL P0 (unsaved-changes guard, rotating backups + restore, confirmation dialogs, undo/redo)
- [ ] **M8** — release 1.0.0 (version, CHANGELOG/README, AppImage, GitHub Release on tag)
- [ ] Human: in-game validation (blank save × six classes, platform conversion, profile keys, class change)

---

## 2. Roadmap

| Milestone | Scope | State |
|---|---|---|
| M3 | Backpack/Bank match the mockup | ✅ done (baselines provisional) |
| **M4** | All other screens except Skills → Homestead, contrast 0/0 | not started |
| **M5** | Skills redesign (rules + board, details/keyboard, respec/build string) | not started (drop-in staged) |
| **M6** | Consistency lock: enable `tests/lint_tokens.rs`, ratchet → 0, byte-exact theme switch | not started |
| **M7** | QoL P0 (data safety) | not started |
| **M8** | Release 1.0.0 + packaging | not started |

Post-1.0 (deferred): DLC state editor (field 27/44/45), mementos (32/33), UI-prefs/stats decode,
challenges screen.

---

## 3. M4 — every screen onto Homestead

**Goal:** bring every screen except Skills onto the Homestead component set and make the contrast
gate hard-zero per screen. Skills (M5) keeps its old `Tokens::` code, so the `Tokens::` lint stays
disabled until **M6**. **Dialogs are in scope, after the 13 screens, as separate commits.**

### Definition of done (per screen)
1. `ACCEPTED_FAILURES[screen] == [0, 0]` in `crates/lootsmith-gui/tests/contrast.rs`.
2. No `Tokens::`, no `CornerRadius`, no `card(`, no `theme::{section,muted,faint}`, no `ui.heading`,
   no raw `ui.button`/`ui.small_button`/`ui.checkbox`/`egui::Button`/`egui::Checkbox`/`egui::Frame`,
   no rounded group boxes.
3. Panels/headers/buttons/checkboxes come from `ui/widgets.rs` (`egui::Grid`, `ScrollArea`,
   `DragValue`, `ComboBox`, `TextEdit` remain allowed as layout/input primitives; prefer
   `ui::widgets::{text_field, search_field, number_field, dropdown}`).
4. A **per-screen behaviour test**: edit a field through the UI → save → reload → assert the value
   changed **and nothing else changed** (required in that screen's commit).
5. **`tests/padding.rs`** passes for the screen (extend it to every migrated tab as each lands;
   no text within 10 px of a shell panel border).
6. Snapshot baseline, dark and light, reviewed as PNG.

**Reference pattern:** Backpack/Bank (M3) — `ui/item_table.rs` + `ui/widgets::{panel, ribbon,
button, stat_block, part_row, dropdown, text_field}`.

### Work order (one commit each; ratchet drops with each)
- **M4.0 — the 8 contrast runs (isolated first commit):** World 0/2, Travel 0/1, About 0/1
  (`Tokens::TEXT_DIM`/`ACCENT` on light parchment), Vehicles 1/1 and Missions 1/1 (disabled raw
  `ui.button`). Then set those 5 `ACCEPTED_FAILURES` rows to `[0,0]`.
- **M4.1–M4.10 — General, Character, Currency, Ammo, Raw, World, Travel, Vehicles, Missions, About**
  (World/Travel/Vehicles/Missions fold in their M4.0 fix). Replace `card()`→`panel`,
  `theme::section`→section label, raw buttons/checkboxes→`ui::widgets::{button,checkbox}`,
  `TextEdit`→`search_field`, `ComboBox`→`dropdown`, `ui.strong`→column labels,
  `Tokens::*`→palette (`p.dim`/`p.gold`).
- **M4.11 — dialogs (separate commits):** Settings, Action search, Paste BL2(…), Saves found,
  New save, Profile, Import sections → Homestead modal panels.
- **M4.12 — retire `app/widgets.rs::{card,empty_state,status_dot}` and `theme::{section,muted,faint}`**
  (move `empty_state` to `ui/widgets.rs`). `theme::Tokens` stays until M6.

### Live contrast ratchet (dark/light failing runs)
| Screen | Now | Target | Legacy left |
|---|---|---|---|
| General | 0/0 | 0/0 | `card`, `section`, 2× `ui.button` |
| Character | 0/0 | 0/0 | `card`, `section`, `selectable_label`, `button`, `ComboBox` |
| Currency | 0/0 | 0/0 | `ui.heading`, `ui.label` |
| Ammo | 0/0 | 0/0 | `section`×2, `ui.button`, `ui.strong`×6 |
| Raw | 0/0 | 0/0 | `ui.heading`, `small_button`×2, `ui.strong`×5, `TextEdit`×3 |
| World | 0/2 | 0/0 | `Tokens::TEXT_DIM`×2, `section`×5, `ui.button`×3, `ui.checkbox`, `muted`×2 |
| Travel | 0/1 | 0/0 | `Tokens::TEXT_DIM`, `small_button`×2, `ui.button`, `strong`, `muted` |
| Vehicles | 1/1 | 0/0 | `ui.button`, `selectable_label`×2, `ComboBox`, `strong` |
| Missions | 1/1 | 0/0 | `ui.button`, `CollapsingHeader`, `section`, `selectable_label`, `strong`×3 |
| About | 0/1 | 0/0 | `Tokens::ACCENT`, `card`, `section`, `muted`×4, `faint` |
| Backpack/Bank | 0/0 | 0/0 | — (M3 reference) |
| Skills | 24/28 | — | **M5** |

---

## 4. M5 — Skills redesign

Drop-in in `~/Downloads/Skills redesign/` (`skill_rules.rs`, `skill_board.rs`) targets
`crates/lootsmith-gui/src/skill_rules.rs` and `ui/skill_board.rs`; replace `app/tabs/skills.rs`;
delete `tree_card`/`skill_tile`/`Tint`… from `app/widgets.rs`.

**Prerequisite P0 (before any replacement):** `skills.json` has **no per-skill column data** and the
count rule (`1→centre, 2→flanks, 3→row`) is wrong for **18 of 108 tiers**. Columns **are** in the
source page: `bl2skills.com` marks a skill's column with class `push1` (`.skill{width:52px;float:left}`,
`.tree{width:174px}`, `.push1{margin-left:58px}` → `col_i = i + Σ push_0..i`). Also two **phantom
skills** named `\1` (icon `class-tree-\i(0)`) are already committed (Salvador Brawn T6, Maya Cataclysm
T6) — filter them. Tasks: extend `scripts/gen-skill-data.py` to emit `col` + drop placeholders and
bump `version` to 2; regenerate + diff; make `skill_rules::slots` column-aware with a count fallback
and tests; show the 18 ASCII tree grids for human comparison.

**Milestones:** M5a rules + board + `tabs/skills.rs` + parity tests (1–5); **M5b** details pane,
keyboard, shift fill/clear, "ignore budget" switch, zero contrast at 1x/2x/3x — **REVIEW STOP**;
**M5c** Respec + `LS2:<class>:<digits>` build string + Other-skills panel.

---

## 5. Review bundle

**Script:** `scripts/review-bundle.sh`. **Output:** `artifacts/bundles/lootsmith-review-<short-hash>.zip`
(create the dir if missing; `/artifacts/` is gitignored). **Keep the newest 5 bundles.**

**Fail the script if any check fails:** tree clean and README hash == HEAD · no pure-black (0,0,0)
pixel at screenshot corners/edges · every screenshot newer than the HEAD commit · no
`tests/snapshots/review` file and no `*.old.png` in the zip · zip **< 5 MB**.

**Contents:**
1. `README.txt` — HEAD hash + subject, date, milestone just finished; test line; clippy/fmt result;
   commits since the previous bundle; **files deleted** since the previous bundle; contrast ratchet
   table; known mockup differences (numbered); open issues.
2. `screenshots/` — every screen, dark+light, **100% and 200%**, rendered from current code
   (`<screen>_<theme>_<scale>.png`); Backpack side-by-side with the mockup (mockup | mine 100% |
   mine 200%), dark+light; a contact sheet per theme; the mockups from `docs/design/`.
3. `src/` — tracked source only, no assets (exclude `crates/*/assets/skills`, portraits, icons,
   `data/*.json`, `target/`, `.git`); include `.rs`, `Cargo.*`, docs, `scripts`, `AGENTS.md`, spec
   files, tests (not snapshot PNGs).
4. `diff.patch` — `git diff <previous-bundle-hash>..HEAD`, source files only.

---

## 6. Key facts & gotchas (hard-won)

**Two compression layers** — `File = SHA1(20) || u32BE(inner_len) || compress(Inner)`,
`Inner = u32BE(size) || "WSG" || u32 version=2 || u32 crc32(proto) || i32 proto_size || Huffman(proto)`.
1. Huffman is not byte-reproducible → gate on protobuf byte-identity + `huffman_stable()`.
2. Outer compressed bytes differ (MiniLZO vs lzokay) → gate on the inner buffer.
3. Item blobs: real-content blobs are byte-identical; blank 40-byte placeholders only semantically stable.

**profile.bin**: SHA1(20) over the rest · u32BE size · LZO1X · entries
`[kind][id BE u32][dataType][value][0x00]` (1=Int32BE, 4=String, 5=Float, 6=Binary, 8=Int8).
Ids 136 rank / 138 tokens / 143 stats / 162 keys / 300 unlocks; the entry table is the byte gate.

**Field 13 `InventorySlotData`** is a single required message; field 38 `ChallengeList` is repeated
(132 in the fixture). Field-15 `StatsData` holds challenge progress (`u32 4 || u32 size || u16 count`
+ 12 bytes/entry). OP entries (40) must not be synthesized without a verified ContentId.

**Dumps carry a UTF-8 BOM** — strip `\u{feff}` before serde_json.

**GUI/egui 0.36:** `eframe::App::ui(&mut self, ui, frame)`; `egui::Panel::*` into the root Ui;
`ComboBox::from_id_salt`; tab modules `use crate::app::*;`. Tab-bar labels include counts —
kittest queries must match exactly.

**Homestead spec rules** (`LOOTSMITH_UI_SPEC.md`): flat only (no shadow/gradient/rounded corners);
hover/focus change borders only; integer UI scale (Auto / 1x–4x); WCAG ≥ 4.5:1 (non-text 3:1).

**Game data is not bundled** (bundled tables under `crates/lootsmith-core/assets/data/` are the
documented exception; a picked folder overrides them). Rarity has no dump field — classify from
`assets/rarity.json`, never guess.

**In-game edits the game clamps:** SDU levels feed slot maximums at load (backpack = 12+3·lvl,
bank = 6+2·lvl); `apply_form` writes `MaxBankSlots` from the form; `refresh_sdus` keeps
`form.bank_slots` synced.

---

## 7. Build / test / run / automate

```sh
cd ~/Projects/lootsmith
cargo build --release
cargo clippy --workspace --all-targets -- -D warnings   # must stay clean
LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace
cargo run -p lootsmith-cli -- inspect|verify|dump|items|skills|travel|vehicles|missions|ammo|sdus|challenges|world <save>
cargo run -p lootsmith-cli -- edit <save> --max-sdus / --backpack-slots 39 / --bank-size 24
cargo run -p lootsmith-gui                       # window
cargo run -p lootsmith-gui --example snapshot -- /tmp/opencode/shots
UPDATE_SNAPSHOTS=1 cargo test -p lootsmith-gui --test snapshots   # then OPEN the PNGs
scripts/review-bundle.sh                         # build the milestone review bundle
```

Testing reality: the GUI suite runs on the **synthetic** save (`lootsmith_core::fixture`), never a
personal fixture; dumps-dependent tests fail loudly under `LOOTSMITH_REQUIRE_FIXTURES=1`.
Snapshot baselines live in `crates/lootsmith-gui/tests/snapshots/`.

---

## 8. History (newest first)

`298bef3` M3: parts union + editable stat blocks · `a9a9948` workbench balance parts · `61262b5` M3(d)
sidebar/status/elision · `532a29d` M3(c) workbench + Ctrl+K · `1e05d3c` README/CHANGELOG · `60fa1ea`
1.0 roadmap · `61e869c` list fixes · `bb6c772` black margins + padding · `177016e` mockup item list ·
`2af08bd` auto UI scale · `3f3da65` Homestead shell · `8f7ba1d` widget set · earlier phases as in
git log.
