# Lootsmith — Plan, Status & Handoff

A cross-platform Borderlands 2 save editor written in Rust.

**Repo:** `~/Projects/lootsmith` · **HEAD:** `6a9733c` (working tree clean)
**Updated:** 2026-10-10 · **Reference:** Gibbed's Borderlands 2 Save Editor (C#, zlib)
· upstream `github.com/gibbed/Gibbed.Borderlands2` · agent brief at repo root: **`AGENTS.md`**

> Single source of truth after a context compaction. Read §1 (checklist), §2 (the agent brief
> rules + gap table), §3 (commands) and §4 (facts/gotchas) first.

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
- [x] `partlists`: Balance Part Lists (`<balance>.PartList`, Complete/Selective) → constrained pickers
- [x] `nameparts`: Weapon/Item Name Parts + Items.json → item display names
- [x] `fixture`: `synthetic_save()` so CI runs without a personal save
- [x] Ammo pools: `ResourceData` (field 11) typed `Resource` + `mutate_resource` (**gap 1**)
- [x] SDUs: `BLACK_MARKET_KEYS`/`SDU_LEVEL_MAX`, `black_market_upgrades`, `set_sdu` (rewrites
      `InventorySlotData` = 12+3·lvl and `MaxBankSlots` = 6+2·lvl), `max_all_sdus` (**gap 2**)

### CLI (`lootsmith-cli`, binary `lootsmith`) — done
- [x] `inspect`, `verify`, `dump`, `items`, `skills`, `travel`, `vehicles`, `missions`, `saves`
- [x] `ammo`, `sdus`, `new --class NAME`, `json`/`unjson`, `profile`
- [x] `edit`: level/points/currency/golden keys/bank slots · `--item/--part/--set` · `--skill` ·
      travel · `--steering-mode` · missions · `--ammo/--amount/--sdu` · `--market NAME --sdu-level L` ·
      `--backpack-slots S` / `--bank-size S` / `--max-sdus` · `--out`, `.bak`, re-verify

### GUI (`lootsmith-gui`) — full editor, 12 tabs, split into modules
- [x] `app/mod.rs` + `app/widgets.rs` + `app/item_editor.rs` + `app/tabs/{general,character,currency,ammo,skills,travel,vehicles,missions,backpack,raw,about}.rs`
- [x] Toolbar: Open / Find saves / Save / Save As / Reload / New… / Import… / Profile… / platform combo / Game data
- [x] Tabs: General, Character, Currency, Ammo, Skills, Fast Travel, Vehicles, Missions, Backpack (n), Bank (n), Raw, About
- [x] Character: class dropdown (DLC flags 44/45) + overpower level + reset; General: Max/OP-max (72/80)
- [x] Ammo tab: SDU table (9 named upgrades, 0..=9/…), Backpack/Bank slot editors, Max all SDUs; ammo pools
- [x] Skills: embedded UCP calculator (pyramid layout, colour fill to 26 pts)
- [x] Fast Travel: DLC-grouped, per-group Check/Uncheck all, star last
- [x] Backpack/Bank: item display names, balance-constrained pickers, Game Stage + manufacturer grade, codes, sync
- [x] Raw: search, scalar + int/list element editing
- [x] New save (clone/blank), section import, profile.bin window; platform Save As
- [x] `tests/synthetic.rs`; snapshot example falls back to the synthetic save

### Remaining
- [ ] Gap 3 Challenges (38/39/40/42) — state, unlocks, prestige
- [ ] Gap 4 World progress (29/30/43/46), Gap 5 DLC state (27), Gap 6 Mementos (32/33)
- [ ] Gap 7 Bulk ops (complete/reset missions, max everything)
- [ ] Gap 8 UI prefs/stats (19/15) — low
- [ ] Gap 9 Game-data onboarding; Gap 10 Packaging; Gap 12 blank-save in-game validation (human)
- [ ] Re-roll `~/Downloads` archive; verify icon after relaunch (human)

---

## 2. Agent brief (AGENTS.md) — rules + gap table

**Per-feature order:** core accessor/mutator → core test → CLI flag → GUI control → GUI test.
Untouched fields re-serialize byte-identically; never rewrite a message you did not edit;
game data stays user-supplied. Field layouts come from upstream, never memory:
`raw.githubusercontent.com/gibbed/Gibbed.Borderlands2/master/projects/Gibbed.Borderlands2.ProtoBufFormats/WillowTwoSave/<Name>.cs`
or the proto mirror `github.com/Oberacda/Borderlands2SaveEditor/borderlands2/protos/*.proto`.

Task template:

```
Task: <one feature from the gap table>
Read first: AGENTS.md, crates/lootsmith-core/src/savegame.rs, upstream <file>.
Scope: core + CLI + GUI + tests for this feature only.
Steps:
  1. Add typed accessor/mutator in lootsmith-core (field numbers from upstream).
  2. Add a round-trip test in crates/lootsmith-core/tests/.
  3. Add CLI flags in lootsmith-cli and list output in `describe`.
  4. Add a GUI tab or section in its own module, with a kittest UI test.
Done when: `cargo test --workspace` passes, an edited save re-parses with only the intended
field changed, and the CLI and GUI both expose the feature.
Do not: touch unrelated fields, rewrite proto.rs, or change container.rs.
```

### Gap table (live status)

| # | Feature | Save field(s) | State |
|---|---|---|---|
| 1 | Ammo pools | 11 `ResourceData` | ✅ `8be3ca9` |
| 2 | SDUs | 36/56/13 | ✅ `6a9733c` |
| 3 | Challenges | 38/39/40/42 | next |
| 4 | World progress | 29/30/43/46 | schema only |
| 5 | DLC state | 27 (+44/45) | import-only |
| 6 | Mementos | 32/33 | schema only |
| 7 | Bulk ops | many | per-DLC travel exists |
| 8 | UI prefs / stats | 19/15 | raw only — low |
| 9 | Game-data onboarding | n/a | open |
| 10 | Packaging | n/a | open (AppImage/Flatpak + CI) |
| 11 | Fixtures | n/a | ✅ `2c53b1b` |
| 12 | Blank-save validation | n/a | human step |

### Verified schemas
- `ResourceData { string Resource=1; string Pool=2; float Amount=3; int32 UpgradeLevel=4 }`
- `PackedItemData { bytes Serial=1; int32 Quantity=2; bool Equipped=3; int32 Mark=4 }`
- `PackedWeaponData { bytes Serial=1; QuickWeaponSlot=2; PlayerMark=3; int32 Unknown4=4 }`, `BankSlot { bytes Serial=1 }`
- `SkillData { string Skill=1; int32 Grade=2; int32 GradePoints=3; int32 EquippedSlotIndex=4 }`
- `OneOffLevelChallengeData { int32 PackageId=1; int32 ContentId=2; repeated uint32 Completion=3 }` — don't synthesize OP entries
- DLC class packages: Mechromancer = Tulip 2, Psycho = Lilac 6 (field 45). No colour field in the save top level.
- SDU order (field 36): rifle, pistol, launcher, shotgun, smg, sniper, grenade, backpack(7), bank(8);
  backpack = 12+3·lvl, bank = 6+2·lvl (pclifford's savefile.py formulas)

---

## 3. How to build / test / run / automate

```sh
cd ~/Projects/lootsmith
cargo build --release
cargo clippy --workspace --all-targets -- -D warnings   # must stay clean
cargo test --workspace                                  # 97 tests
cargo run -p lootsmith-cli -- ammo|sdus|items|skills|travel|vehicles|missions <save>
cargo run -p lootsmith-cli -- new/json/unjson/profile/…
cargo run -p lootsmith-cli -- edit <save> --ammo 5 --amount 999 --sdu 3
cargo run -p lootsmith-cli -- edit <save> --max-sdus / --backpack-slots 39 / --bank-size 24
cargo run -p lootsmith-gui                       # window
cargo run -p lootsmith-gui -- --selftest --open <save>
cargo test -p lootsmith-gui                      # kittest, by label
cargo run -p lootsmith-gui --example snapshot -- /tmp/opencode/shots
```

Paths: fixture `tests/fixtures/pc_Save0001.sav` (gitignored); real save
`~/bl2-zluda/…/SaveData/76561198098137537/Save0001.sav`; profile
`~/.local/share/aspyr-media/borderlands 2/willowgame/savedata/76561198098137537/profile.bin`;
dumps `~/Projects/bl2-rogue/data/gibbed/`; launcher `~/.local/share/applications/lootsmith.desktop`.

---

## 4. Key facts & gotchas (hard-won)

**Two compression layers** — `File = SHA1(20) || u32BE(inner_len) || compress(Inner)`,
`Inner = u32BE(size) || "WSG" || u32 version=2 || u32 crc32(proto) || i32 proto_size || Huffman(proto)`.
1. Huffman is not byte-reproducible → gate on protobuf byte-identity + `huffman_stable()`.
2. Outer compressed bytes differ (MiniLZO vs lzokay) → gate on the inner buffer.
3. Item blobs: real-content blobs are byte-identical; blank 40-byte placeholders only semantically stable.

**profile.bin**: SHA1(20) over the rest · u32BE size · LZO1X · entries `[kind][id BE u32][dataType][value][0x00]`
(1=Int32BE, 4=String, 5=Float, 6=Binary, 8=Int8). Ids 136 rank/138 tokens/143 stats/162 keys/300 unlocks.
The entry table is the byte gate; the container hash is recomputed on write.

**Field 38 `ChallengeList` is a single required message**, not a list. Field 13 `InventorySlotData`
is a single required message. OP entries (40) must not be synthesized without a verified ContentId.

**Dumps carry a UTF-8 BOM** — strip `\u{feff}` before serde_json.

**GUI/egui 0.36:** `eframe::App::ui(&mut self, ui, frame)`; `egui::Panel::*` into the root Ui;
`ComboBox::from_id_salt`; tab modules import `use crate::app::*;` (their `super` is `tabs`);
tab-bar labels include counts ("Backpack (14)") — kittest queries must match exactly.

**Game data is not bundled** (calculate skill data is the documented exception).

**In-game edits the game clamps:** SDU levels feed slot maximums at load (12+3·lvl / 6+2·lvl);
`apply_form` writes MaxBankSlots from the form — `refresh_sdus` keeps `form.bank_slots` synced.

---

## 5. Working rules for the next agent

1. Work the gap table, one gap per commit, following the task template.
2. After each change: tests + clippy + release build + `--selftest` + snapshot.
3. Commit messages name the gap (`… (AGENTS.md gap N): …`).
4. Keep README + this plan in sync; mirror to `docs/PLAN.md`.
5. Anything needing the real game is a human step — say so.

---

## 6. Phases

| Phase | Scope | State |
|---|---|---|
| 0–5 | core + CLI | ✅ |
| 6 | GUI | ✅ 12 tabs |
| 7 | AGENTS.md gaps | 1, 2, 11 + split done; challenge/next: 3 |

---

## 7. History (newest first)

`6a9733c` SDUs (gap 2) · `8be3ca9` ammo (gap 1) · `2c53b1b` fixture (gap 11) · `0ad078a` split ·
`2b4e26e` AGENTS.md · `2192c08` names/class/OP/travel · `83fb0cb` balance pickers ·
`bd216a8` profile.bin · `53c3ae5` raw lists · `031603f` new/import/JSON/platform ·
`9f5184a` skill plates · `64af860` 3×6 grid · `d1175a9` calculator composition ·
`cdd1e64` calculator polish · `e692c58` skill calculator · `b0cd3fe` GUI redesign ·
`ebcce76` icon · `84a1f2c` discovery · earlier phases (0–6) as in git log.

Archives: `~/Downloads/lootsmith-0.0.1-2192c08.tar.gz` — predates the last 7 commits; re-roll on request.
