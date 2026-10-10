# Lootsmith: UI Workflow Brief (autonomous testing)

Read this before any change under `crates/lootsmith-gui/`. It applies on top of `AGENTS.md` (project brief) and `LOOTSMITH_UI_SPEC.md` (target design).

## 1. Principle

Every UI change must be verified by running tests, not by inspection. The harness is `egui_kittest`: it runs the real UI (`LootsmithApp::build`) in memory, finds widgets by label through AccessKit, clicks them, and renders to PNG. No window, no focus, no display server.

## 2. Fix these first (they make green results meaningless)

1. **Tests skip silently.** `tests/ui.rs` returns early with "fixtures missing; skipping" when the save or dumps are absent, and the test still passes. On a checkout without fixtures, `cargo test` is green and tests nothing.
   - Add a helper `require_fixtures()` that panics when `LOOTSMITH_REQUIRE_FIXTURES=1` is set and the files are missing. Use it in every fixture-dependent test.
   - Always run the suite with `LOOTSMITH_REQUIRE_FIXTURES=1`.
2. **Fixtures live outside the repo.** The save is `tests/fixtures/pc_Save0001.sav` (git-ignored) and the dumps default to `../../../bl2-rogue/data/gibbed/Asset Library Manager.json`, a path outside the package. Replace both:
   - Generate a synthetic save in test setup with `SaveGame::blank` plus a few items, written to a temp directory.
   - Point tests at a documented dumps folder through `LOOTSMITH_ASSET_LIBRARY` and fail loudly when it is unset.
3. **No snapshot baselines exist.** `examples/snapshot.rs` renders PNGs but compares nothing. Add real comparison tests (section 4).

## 3. The loop (every UI task)

```
1. Write or update the test first (widget behavior and/or snapshot).
2. Run it and confirm it fails for the right reason.
3. Implement the change.
4. cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
5. LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace
6. If a snapshot differs: open the new PNG and the stored one, decide whether the change is intended.
   Intended: UPDATE_SNAPSHOTS=1 cargo test -p lootsmith-gui, then re-run without it.
   Not intended: fix the code.
7. Report: tests run, tests passed, snapshots changed (list each with the reason).
```

## 4. Test types

### Behavior tests (widget level)
- Build with `Harness::builder().with_size(...).build_ui_state(|ui, app| app.build(ui), app)`, as `tests/ui.rs` does.
- Find widgets by label (`harness.get_by_label("Backpack")`), call `.click()`, call `harness.run()`, then assert on `harness.state()`.
- Every new control needs one test that changes state through the UI and one that saves, reloads the file from disk and checks the value.
- Give every new interactive widget an accessible label. A widget without one cannot be tested.

### Snapshot tests (visual regression)
- `harness.snapshot("name")` compares against `tests/snapshots/name.png` and fails on any pixel difference. `egui_kittest` is already built with the `snapshot` and `wgpu` features.
- Cover every screen in both themes at 100% and 200% scale: `backpack_dark_1x`, `backpack_light_2x`, and so on.
- Use a fixed window size (1280 x 800) and the synthetic fixture so output is deterministic. Disable animations and cursor blink in the harness.
- Keep the diff threshold at zero. The pixel style makes any drift visible, which is the goal.
- Commit baselines. A snapshot PNG changed by a commit must be listed in the report with its reason.

### Geometry and contrast tests (pure functions)
- `notched_rect(rect, 4.0)` returns 12 points with corners stepped at 4 px.
- Contrast for every palette text/surface pair is at least 4.5:1, in both themes.
- Layout constants (sidebar 204 px, top bar 56 px, status bar 34 px, panel gap 16 px) are asserted against rendered widget rectangles.

### Rule tests (the design's hard rules)
- A test that scans the shapes of a rendered frame and fails on any rounded rectangle, shadow, gradient mesh or non-integer border width.
- A test that sets scale to 150% and expects the app to clamp to 100% or 200%.

### Behavior tests for QoL features
- Close with unsaved changes shows the Save / Discard / Cancel dialog.
- Undo then redo restores the exact `SaveGame` bytes.
- Drag-and-drop of a `.sav` opens it.
- The eleventh save leaves exactly ten backups.
- Action search (Ctrl+K) dispatches the chosen action.

## 4a. Hard stops

1. Never edit a test or delete a snapshot to make a failure disappear. If the test is wrong, say why in the report and change it in its own commit.
2. Never accept `UPDATE_SNAPSHOTS=1` output without opening the PNGs.
3. Never mark a task done while any test is skipped or ignored. Report the skip count and the reason.
4. Never change `crates/lootsmith-core` format code (`proto.rs`, `container.rs`, `huffman.rs`) during UI work. If a UI task needs a core change, stop and report.
5. If the same test fails three times after changes, stop and report what was tried.

## 5. Where humans still look

Tests prove geometry, contrast, behavior and absence of drift. They do not judge whether the screen feels right. Before accepting a new baseline for a screen, the human reviews its snapshot PNGs in both themes. The agent lists the PNG paths in its report.

## 6. Report template (end of every task)

```
Task: <name>
Changed: <files>
Tests: <run> run, <passed> passed, <failed> failed, <skipped> skipped
Snapshots changed: <name: reason> (none if empty)
New snapshots: <names>
PNGs to review: <paths>
Open issues: <anything not done>
```
