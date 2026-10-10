# Lootsmith: Visual Overhaul and QoL Spec ("Homestead")

Companion to `AGENTS.md` (the project brief). Same rules apply: core first, tests for every change, one item per task.
This spec replaces the current look entirely. Nothing in `theme.rs` needs to be preserved.
The approved reference is the published mockup (Backpack screen, dark and light). Match it.

## 1. Problems in the current GUI (from source)

1. **The light theme is broken.** `theme::apply` selects `Visuals::light()` or `Visuals::dark()`, then overwrites panel, window, widget and text colors with the dark `Tokens` constants regardless of theme.
2. **11 tabs in one top row.** This does not scale once Ammo, SDUs, Challenges and World are added.
3. **Default egui fonts at 13 px body / 11 px small**, with no UI scale setting.
4. **Setup relies on raw text fields** for the save path and the game-data folder.
5. **No protection against data loss:** no unsaved-changes prompt on close (no `close_requested` handling), no undo or redo, no confirmation for destructive actions, a single `.bak` that the next save overwrites.
6. **Only four shortcuts** (Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+R).
7. **Challenges are an unfiltered text list**; the Raw tab is the only place to edit many fields.

## 2. Visual direction

Name: **Homestead**. A cozy workshop-cabin look in pixel-art style, drawn from the Lootsmith logo: wood, parchment, stitching, gold rivets, banner ribbons, treasure-map trails.

### 2.1 Hard rules

1. **Flat only.** No shadows, gradients, blur, glass, glow or transparency effects. Set `window_shadow` and `popup_shadow` to `Shadow::NONE`.
2. **Nothing floats.** Panels sit in a grid with 16 px gaps. Popups and dialogs are modal, bordered and flush-centered, never drop-shadowed.
3. **Hard pixel edges.** No rounded corners anywhere (`CornerRadius::ZERO`). "Rounded" shapes are 4 px stepped corners.
4. **Texture is pattern, not shading.** Textures are flat two-color tiles (dither, plank lines). They never change brightness across a surface.
5. **No pure white, no pure black.** Light theme uses parchment; dark theme uses warm black.
6. **Hover and focus change borders, not fills.** Hover: border becomes `gold`. Focus: add a 2 px `acc` outline.
7. **Whole physical pixels.** UI scale offers 100%, 150% and 200% only. 150% is
   *not* an integer multiple (1.5x), so a value authored in points can land on a
   half physical pixel. Every painted rect, line and stroke is therefore snapped
   to whole physical pixels (`ui::pixel::{snap, snap_pos, snap_rect, snap_len}`),
   and a test renders the shapes at all three scales and asserts every
   tessellated vertex is whole (`ui::pixel::tests`).
   Dimensions that are **odd at 150%** and how they are snapped:
   - the 3 px dotted trail divider — 4.5 px, rounded up to 5 px;
   - 13 px and 15 px text — 19.5 / 22.5 px: egui rasterises glyphs at the device
     scale, and text origins are snapped so baselines stay on whole pixels;
   - the 14 px rarity diamond — 21 px exactly, but its **centre** is snapped so
     the 2 px outline stays crisp;
   - every panel/bar edge and 1–4 px stroke width — snapped independently.

### 2.2 Palettes (two complete token sets; never mix)

| Token | Dark | Light | Use |
|---|---|---|---|
| `bg` | `#14100C` | `#CDBB92` | window background (textured) |
| `pan` | `#1E1812` | `#E8DDBF` | panel fill |
| `fg` | `#FFC98A` | `#3A2616` | body text |
| `dim` | `#C29862` | `#523D25` | secondary text (light darkened for 5.42:1 on `bg`) |
| `acc` | `#FFB866` | `#A8470F` | selected fill, primary button |
| `on` | `#14100C` | `#F6EBCF` | text on `acc` |
| `gold` | `#E3B04B` | `#5E3E06` | trim, section labels, hover border (light darkened for 5.13:1 on `bg`) |
| `wd` | `#5A3A20` | `#8A5A32` | wood: panel border, bars |
| `wd2` | `#4A2F1A` | `#7A4E2B` | plank line |
| `onw` | `#FFD9A8` | `#F6EBCF` | text on wood |
| `rib` | `#D9A25A` | `#A8470F` | ribbon fill |
| `onr` | `#14100C` | `#F6EBCF` | text on ribbon |
| `brd` | `#8C6A40` | `#8A6A44` | stitching, dotted dividers (>=3:1 on `pan`) |
| `tex` | `#1A150F` | `#C4B186` | background dither tile |
| `wdb` | `#96714A` | `#8A5A32` | wood *border* on `pan` (>=3:1); `wd` stays the wood fill |
| `dng` | `#FF8A70` | `#9E2B14` | destructive text |

The light `gold` value is darker than in the mockup (`#8C5F0E`) because that value is estimated at about 4.1:1 on `pan`, below the 4.5:1 target for small label text.
Rules: `theme::apply(ctx, theme)` takes a `Palette` struct chosen by `theme`; no color constants are referenced outside it. Add a unit test that computes WCAG contrast for every text/surface pair (`fg/pan`, `dim/pan`, `gold/pan`, `on/acc`, `onw/wd`, `onr/rib`, `dng/pan`) and fails below 4.5:1. If a pair fails, darken or lighten the token until it passes.

### 2.3 Item rarity

Rarity appears as a flat diamond (14 px) next to the item name, plus the rarity written in text on the editor. Color is never the only signal.

Rarity diamonds are drawn with a **2 px `fg` outline** on every tier, so the
White diamond stays visible on light parchment (its fill alone measures well
below 3:1 there). The outline is part of the rule, not an accident.

| Rarity | Diamond |
|---|---|
| White | `#D8DAE0` |
| Green | `#5ED08A` |
| Blue | `#4C9BFF` |
| Purple | `#B06CFF` |
| E-tech | `#FF4FA3` |
| Legendary | `#FF8A1F` |
| Pearlescent | `#3DE0D0` |

Derive rarity from the balance definition: `items::rarity(balance) -> Rarity` in core, driven by the dumps.

### 2.4 Typography

- UI: **Pixelify Sans** (OFL), 15 px body, 13 px small caps-style labels, 18 to 24 px headings. Weights 400, 600, 700.
- Numbers and item codes: **VT323** (OFL), 19 px (26 px for large stat values).
- **Every digit run uses VT323** through `ui::widgets::num()` — ids, counts,
  levels, ranks, quick-slot and slot counts — never the label font (Pixelify
  Sans' `5` reads as `S`). Mixed labels (e.g. `Backpack (5)`) set the count in
  VT323 via `ui::widgets::counted_label`. `tests/digits.rs` renders `0123456789`
  in `num()` and asserts every glyph is distinct.
- Embed both with `include_bytes!` and register them through `egui::FontDefinitions` in `main.rs`; set `TextStyle::Body`, `Button`, `Heading`, `Small`, `Monospace` explicitly.
- Disable font hinting changes and keep text positions on whole pixels (`Visuals::text_cursor`, `pixels_per_point` integer multiples).

### 2.5 Drawing primitives (egui implementation, in `ui/pixel.rs`)

| Primitive | How |
|---|---|
| Notched rect (4 px stepped corners) | `notched_rect(rect, step) -> Vec<Pos2>` (12-point polygon); draw with `Shape::convex_polygon` for fill and `Shape::closed_line` for the border |
| Panel | Notched rect, 4 px `wd` border, `pan` fill, plus a 2 px dashed `brd` "stitch" line inset 10 px (`Shape::dashed_line`, dash 6, gap 4) |
| Ribbon (section title) | Polygon with swallow-tail ends (10 px notch both sides), drawn as two convex halves so the notches are transparent; `rib` fill, `onr` text, 5 px vertical padding, 26 px horizontal padding |
| Plank bar (top bar, status bar) | `wd` fill with a 2 px `wd2` **horizontal** seam every 26 px (the mockup's `repeating-linear-gradient(180deg, …)`); 4 px `gold` rule on the inner edge |
| Rivet | 8 x 8 px `gold` square |
| Trail divider | 3 px dotted `brd` line (`dashed_line`, dash 3, gap 5) |
| Hatch | 45° diagonal `tex` lines, 2 px wide on a 7 px period (the mockup's `repeating-linear-gradient(45deg, tex 0 2px, transparent 2px 7px)`) |
| Gem | 4-point diamond polygon, flat fill |
| Dither background | 8 x 8 px two-color checker tile, generated at startup as a `ColorImage`, uploaded with `TextureOptions::NEAREST` and `wrap_mode: Repeat`, painted over the central panel rect with a large UV range |
| Logo | `Lootsmith.png` loaded as a `NEAREST` texture, drawn at 96 x 96 in the sidebar |

All widgets are custom-painted wrappers over `ui.allocate_response`, so hit-testing, focus and AccessKit labels stay intact.

### 2.6 Layout (1280 x 800 reference)

1. **Top plank bar** (56 px): rivet, Lootsmith ribbon, file-name chip, unsaved chip, platform label, action search (Ctrl+K), Undo, Redo, Save As, Save (primary).
2. **Sidebar** (204 px, in a panel): logo, then groups separated by dotted trails and gold labels:
   - *Character*: General, Character, Skills, Currency
   - *Inventory*: Backpack, Bank, Ammo & SDUs
   - *Progress*: Missions, Challenges, Fast Travel, World
   - *Account*: Profile, Vehicles
   - *Tools*: Import, Raw
   Rows are 28 px; the active row is a solid `acc` block. Item counts show on the right.
3. **Main column:** a header panel (ribbon title, summary text, filter field, primary actions), then two panels side by side.
4. **Item screens (Backpack, Bank):** left panel 520 px with a table (rarity diamond, name, type, level, mark; 40 px rows, dotted row dividers, solid `acc` selected row) and an action row (Duplicate, To bank, Sync, Remove). Right panel "Item workbench": name with diamond, type line, four stat blocks (Level, Game stage, Grade, Mark), part rows with a dropdown per slot (Body, Barrel, Sight, Accessory, Material, and so on), and the `BL2(...)` item code with a Copy button.
5. **Status plank bar** (34 px): save path, level, last saved time, and a `acc` chip for game-data status. The chip reads "Game data loaded", "Game data partial" or "Game data missing" and opens the fix dialog on click.
6. **Skills tab:** keep the tree calculator; draw nodes as notched squares, add a point-allocation summary ribbon, a Respec button, and build import/export as a short text string.

### 2.7 Components (in `ui/widgets.rs`, built on `ui/pixel.rs`)

`Panel`, `Ribbon`, `PlankBar`, `NavRow`, `Button` (normal, primary, danger), `Chip`, `StatBlock`, `PartRow`, `GemRarity`, `SearchField`, `TrailDivider`, `ConfirmDialog`, `Toast`, `EmptyState`, `StatusChip`. Every screen uses these; no ad-hoc layout code.

## 3. QoL features (priority order)

### P0: data safety
1. **Unsaved-changes guard.** Intercept `ViewportCommand::CancelClose` on `close_requested` when `dirty`; show Save / Discard / Cancel. Same guard before Open, Reload, New, Import.
2. **Undo and redo (Ctrl+Z, Ctrl+Y / Ctrl+Shift+Z).** Bounded stack (100) of `SaveGame` message snapshots pushed on each committed edit, each labeled ("Set level 80", "Remove item") and shown in the button tooltip.
3. **Rotating backups.** Before every in-place save, copy to `<name>.bak.<YYYYmmdd-HHMMSS>` in a `backups/` folder beside the save; keep the newest 10 (configurable). Add a "Restore backup…" dialog.
4. **Confirmation for destructive actions:** remove item, clear all fast travel, reset missions, new save, import. Specific message ("Remove Hellfire (level 72) from Backpack?").
5. **Pre-save diff.** On Save, show a collapsible summary of what changed (compare `describe()` output before and after, plus item counts). A "don't show again" toggle skips it.

### P1: setup and navigation
6. **First-run wizard:** detect saves (existing discovery code) and game-data folders; show them as selectable panels; state in one sentence which folder is needed.
7. **Drag and drop:** dropping a `.sav` opens it; dropping a folder sets game data (`ctx.input(|i| i.raw.dropped_files.clone())`).
8. **Recent files** menu (last 10, persisted with eframe persistence).
9. **Action search (Ctrl+K):** fuzzy-search every action and tab ("max level", "unlock fast travel", "go to bank").
10. **Global shortcuts:** Ctrl+1..9 for sidebar sections, Ctrl+F focus filter, Ctrl+D duplicate item, Delete remove selected, Ctrl+C / Ctrl+V on selected items copies and pastes `BL2(...)` codes, F5 reload, Ctrl+Z / Y undo and redo.
11. **Validation banner** (a ribbon with fix buttons): level above 72 without OP support, bank slots lower than items stored, item level above character level, skill points above the earned total, named items missing game data.

### P2: item workflow
12. **Multi-select** (Shift/Ctrl click) with bulk actions: move to bank/backpack, delete, sync level, set mark, export codes.
13. **Sort and filter bar:** by type, manufacturer, level range, rarity, mark; saved filters.
14. **Bulk export/import of item codes** as a text file; paste a block of codes to add many items.
15. **Compare view:** two items side by side with changed parts highlighted by a `gold` border.
16. **Inline sanity checks** when editing parts: flag combinations the balance does not allow, instead of silently blocking.

### P3: polish
17. **Toasts** (bordered, docked to the status bar, not floating) for save, copy, import and error results; errors also go to the log with a "Copy details" button.
18. **Tooltips** on every control; a one-line description for each Raw field name.
19. **Keyboard navigation** in lists (arrow keys, Enter to open the workbench, Home/End).
20. **Settings dialog:** theme (System / Dark / Light, following `system_theme`), UI scale (100/150/200%), backup count, confirm-prompt toggles, default platform.
21. **Accessibility:** 2 px `acc` focus outline on every interactive widget, minimum 28 px hit targets, no information conveyed by color alone.
22. **Numeric fields** with thousands separators and a `K / M / B` shorthand parser ("1.5m").

## 4. Implementation order

1. Split `app.rs` into one module per tab (prerequisite; see the project brief).
2. `theme.rs` rewrite with `Palette`; contrast test; fix the light theme.
3. Embed fonts; add UI scale (100/150/200%).
4. `ui/pixel.rs` primitives and the dither texture.
5. `ui/widgets.rs` components.
6. Shell: plank top bar, sidebar, status bar. Migrate one tab at a time; keep UI tests passing after each.
7. P0 QoL items 1 to 5.
8. Item screens (table, workbench, rarity diamonds).
9. P1, P2, P3 items.

## 5. Verification

- `cargo test -p lootsmith-gui` must pass after each step.
- Extend `examples/snapshot.rs` to render every screen in both themes at 100% and 200% scale. Review the PNGs for clipped text, blurry pixel edges and low contrast before merging.
- Add tests: palette contrast for every pair, `notched_rect` geometry (12 points, corners at 4 px steps), close-with-unsaved prompt, undo/redo round trip, drag-and-drop open, backup rotation (10 kept), action-search dispatch.
- Visual acceptance: no shadow, gradient or rounded corner appears in any snapshot; every border is a whole number of pixels.
