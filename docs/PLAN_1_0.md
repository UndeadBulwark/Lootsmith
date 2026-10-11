# Lootsmith 1.0 plan

Roadmap from the current state to the 1.0 release. Milestones from the UI overhaul
(`LOOTSMITH_UI_SPEC.md` §4). Each milestone lands as its own commit(s); every
commit runs the full suite with `LOOTSMITH_REQUIRE_FIXTURES=1`.

## M3 — Backpack/Bank match the mockup  *(in progress)*
- [done] item list: rarity diamond, name, type, level, mark; 40 px rows, dotted
  dividers, solid `acc` selected row (`ui/item_table.rs`).
- [done] list names from `Row.name` (balance fallback, middle-elided); TYPE from the
  weapon type part (`WeaponType_Dahl_Pistol` -> `Pistol`, `Item`, `Unknown`).
- [done] mark diamond only for a real favourite (`PLAYER_MARKS` index 2).
- [done] file-name chip middle-elided; inspector body fully scrollable.
- [ ] (c) item workbench: ribbon, big name + rarity diamond, `Type · Maker · Rarity`
  subtitle, four stat blocks (Level / Game stage / Grade / Mark), PARTS dropdowns,
  ITEM CODE + Copy; header panel (slot count, `Filter items…`, `+ Weapon`,
  `Paste BL2(…)`); action row (Duplicate / To bank / Sync / Remove) under the list.
- [ ] (d) polish: right-aligned sidebar counts, 96 px logo PNG, un-truncated action
  search, no filter/button overlap, middle-elided status path, no right-edge
  clipping; dark/light 1x/2x/3x side-by-side review vs the mockup.

## M4 — All other screens except Skills  *(one batch)*
General, Character, Currency, Ammo & SDUs, Vehicles, Missions, Raw, About, Profile,
Import. Migrate each to `ui/widgets.rs`; a migrated screen must have **zero contrast
failures on arrival**, and no screen may draw its own panel/header/button/checkbox.

## M5 — Skills
Notched tiles, summary ribbon, Respec, the `LS2:` build string, no tints and no
rounded corners.

## M6 — Consistency lock
- Enable `tests/lint_tokens.rs`: ban `Tokens::`, `CornerRadius`, and raw egui
  `Button`/`Checkbox`/`Frame` outside `ui/widgets.rs`; add the rounded-rect rule.
- Drive the **contrast ratchet to 0** on every screen and theme.
- Make the theme-switch test **byte-exact for every screen** (dark/light x 100 %/200 %).

## M7 — Release 1.0.0
- Bump `version` to `1.0.0`; write `CHANGELOG.md`; refresh `README.md` (install,
  usage, screenshots), `docs/PLAN.md` and `AGENTS.md` current-state.
- Packaging: AppImage build; change the CI release job from `upload-artifact` to a
  **GitHub Release** on tag (attach the Linux tarball + AppImage).
- Pre-release gates: full suite `LOOTSMITH_REQUIRE_FIXTURES=1`; the clean-clone
  determinism run; **human in-game verification** (blank save across all six classes).
- Tag `v1.0.0` -> CI builds and publishes.

## Post-1.0 (deliberately deferred)
- **Challenges** screen (no such screen today; not in the mockup) — dropped from
  this release.
- DLC state editor (field 27/44/45), mementos (32/33), UI-prefs/stats decode.
