# Lootsmith: Project Brief for AI Coding Agents

Place this file in the repo root (or name it `AGENTS.md`) and tell your agent to read it before every task.

## 1. Goal

Lootsmith is a Linux-native, Rust replacement for Gibbed's Borderlands 2 Save Editor.
Target: feature parity with upstream Gibbed (https://github.com/gibbed/Gibbed.Borderlands2), plus Linux save discovery and packaging.

Upstream is the source of truth for save-format field layouts. Before implementing any feature, read the matching upstream file (`ProtoBufFormats/WillowTwoSave/*.cs`, `SaveEdit/*`) and copy field numbers and types exactly. Do not guess layouts.

### The milestone gate

`docs/PLAN.md` is the single source of truth for the plan (`~/.opencode/plan/*.md` are pointers to it).

A milestone is **not done** until **all** hold: (1) release build + `LOOTSMITH_REQUIRE_FIXTURES=1
cargo test --workspace` green with no skips; (2) clippy `-D warnings` and `cargo fmt --check` clean;
(3) its **review bundle exists** — `scripts/review-bundle.sh` →
`artifacts/bundles/lootsmith-review-<short-hash>.zip` (built from current code, never copied); and
(4) the report uses the standard template and **ends with the line `Bundle: <full path>`**.

## 2. Architecture rules

```
lootsmith-core   format + data model. No UI, no I/O beyond bytes/files.
lootsmith-cli    thin command layer over core.
lootsmith-gui    egui front end over core. No format logic here.
```

1. Every feature lands in this order: core accessor/mutator -> core test -> CLI flag -> GUI control -> GUI test.
   For anything under `crates/lootsmith-gui/`, also read `LOOTSMITH_UI_SPEC.md` and
   `LOOTSMITH_UI_WORKFLOW.md` (target design + the autonomous testing loop).
2. Untouched fields must re-serialize byte-identically (the `proto.rs` raw-bytes design). Never rewrite a message you did not edit.
3. Every setter has a round-trip test: load fixture -> edit -> `to_bytes` -> `parse` -> assert value -> assert all other fields unchanged.
4. Game data (asset library, parts, names) is user-supplied JSON; the community
   dump tables are additionally bundled under `crates/lootsmith-core/assets/data/`
   (data only, no art or binaries) so a fresh run resolves names with no setup.
   A folder the user picks always overrides the bundled copy. Do not add new
   Gearbox art. Rebuild the bundled tables with `scripts/refresh-data.sh`
   (sources, revisions and licences in `assets/data/PROVENANCE.md`).
5. `app.rs` was split in `0ad078a`: the app now lives in `app/mod.rs` (state, build,
   toolbar, dispatch, model), `app/widgets.rs`, `app/item_editor.rs` and one module per tab
   under `app/tabs/`. Keep it that way — new screens go in their own tab module.

## 3. Current state (verified from source, HEAD `297d032`)

Implemented: container read/write (LZO/zlib, SHA1, CRC32, Huffman), all 58 top-level field specs,
CLI (`inspect verify dump items skills travel vehicles missions saves new json unjson profile ammo sdus
challenges world edit`), GUI tabs (Character, Currency, **Ammo**, **World**, Skills, Fast Travel,
Vehicles, Missions, Backpack, Bank, Raw, Profile, Import, About), item part editing with balance-constrained pickers,
item display names, `BL2(...)` item codes, profile.bin editing, new-save creation (clone + experimental
blank), section import, platform conversion, save auto-discovery (native, Steam/Proton, Wine), game-data
onboarding, tag-triggered CI release + an AppImage builder.

Gaps 1 (ammo), 2 (SDUs), 3 (challenges), 4 (world), 5 (DLC state), 6 (mementos), 7 (bulk ops),
9 (onboarding), 10 (packaging) and 11 (test fixture) are **done**. Gap 8 (UI-prefs/stats decode) is
deliberately left — low priority, needs a use case. Gap 12 (blank-save in-game validation) is a human step.

Testing reality: the GUI suite runs on the **synthetic** save (`lootsmith_core::fixture`), never on a
personal fixture, and dumps-dependent tests fail loudly under `LOOTSMITH_REQUIRE_FIXTURES=1` instead of
skipping. Snapshot baselines live in `crates/lootsmith-gui/tests/snapshots/`.

## 4. Gaps (priority order)

Each gap below is a field that exists in `TOP_LEVEL` in `savegame.rs` but has no typed accessor, CLI flag, or GUI control.

| # | Feature | Save field(s) | Current state | Work |
|---|---|---|---|---|
| 1 | Ammo pools | 11 `ResourceData` | Schema only | Accessors for resource path, pool path, amount, level. Ammo tab with per-weapon-type max/current. "Fill all ammo" button. |
| 2 | Storage Deck Upgrades (ammo, backpack, bank SDUs) | 36 `BlackMarketUpgrades`, 56 `MaxBankSlots`, 13 `InventorySlotData` | 56 is editable; 36 and 13 are schema only | SDU table (name -> index -> level). "Max all SDUs" button. Backpack slot count editor. Keep 56 consistent with bank SDU level. |
| 3 | Challenges | 38 `ChallengeList`, 39 `LevelChallengeUnlocks`, 40 `OneOffLevelChallengeCompletion`, 42 `NumChallengePrestiges` | Names listed read-only; 42 getter only | Per-challenge state editing, level-challenge unlocks, prestige count editor. |
| 4 | World progress | 29 `RegionGameStages`, 30 `WorldDiscoveryList`, 43 `LockoutList`, 46 `FullyExploredAreas` | Schema only | Read/list/edit. Bulk "mark all explored". Lockout reset (raid boss and Crawmerax timers). |
| 5 | DLC state | 27 `DLCExpansionData`, 44/45 DLC class flags | Import-only (27); class flags set via Character tab | List per-DLC flags (`Id`, `ShowNewNotification`, `EnabledState`), editor. |
| 6 | Mementos | 32 `WeaponMementos`, 33 `ItemMementos` | Schema only | List and clear. Needed so removed items do not re-appear in the UI. |
| 7 | Bulk operations | Many | Not present | "Unlock all fast travel" exists per DLC group; add "complete all missions", "reset all missions", "max everything" as single actions. |
| 8 | UI preferences, stats | 19 `UIPreferences`, 15 `StatsData` | Raw bytes only | Low priority. Decode only if a use case appears. |
| 9 | Game-data onboarding | n/a | User must find and point at Gibbed dump JSONs | Detect dump folders next to the binary, under `~/.local/share/lootsmith`, and in the Steam/Wine prefix. Show a first-run dialog that says exactly which folder to select. |
| 10 | Packaging | n/a | Dev launcher script only | AppImage or Flatpak build. CI that runs `cargo test` and builds a release artifact. |
| 11 | Test fixtures | n/a | Tests need `tests/fixtures/pc_Save0001.sav`, which is git-ignored and not in the archive | Add a small synthetic fixture save generated by `SaveGame::blank` so CI can run without a personal save. |
| 12 | Blank-save reliability | n/a | README marks `new` blank saves experimental | Validate in-game for all six classes, then remove the label. |

The plan file lives in the repo at `docs/PLAN.md` (mirrored from `~/.opencode/plan/LOOTSMITH_PLAN.md`).

Rarity has **no field in the dumps** (verified across `Weapon/Item Balance.json`, `Items.json`,
`Weapon Types.json`). It is classified by `crates/lootsmith-core/assets/rarity.json` (data, not code);
unmatched names are `Rarity::Unknown` and must never be guessed — the report is
`~/Downloads/lootsmith-rarity-report.md`.

## 5. Task template (copy per task)

```
Task: <one feature from section 4>
Read first: AGENTS.md (this file), crates/lootsmith-core/src/savegame.rs, upstream <file>.
Scope: core + CLI + GUI + tests for this feature only.
Steps:
  1. Add typed accessor/mutator in lootsmith-core (field numbers from upstream).
  2. Add a round-trip test in crates/lootsmith-core/tests/.
  3. Add CLI flags in lootsmith-cli and list output in `describe`.
  4. Add a GUI tab or section in its own module, with a kittest UI test.
Done when: `cargo test --workspace` passes, an edited save re-parses with only the intended field changed, and the CLI and GUI both expose the feature.
Do not: touch unrelated fields, rewrite proto.rs, or change container.rs.
```

## 6. Why your AI misreads requests

1. The request "make it like Gibbed" has no testable definition. Replace it with section 4 items, one per task.
2. One 4,287-line `app.rs` exceeds what an agent keeps in context. Split it.
3. Field layouts exist only in the upstream C# files. Tell the agent to open them; it cannot recall them reliably.
4. No acceptance criteria. Every task needs the "Done when" line above.

## Agent skills

### Issue tracker

Issues live as GitHub issues in `UndeadBulwark/lootsmith`, driven by the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles map to `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
