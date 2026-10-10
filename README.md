# Lootsmith

A cross-platform save editor for Borderlands 2, written in Rust.

> Lootsmith is a Rust port based on Gibbed's Borderlands 2 Save Editor by Rick
> Gibbed (zlib license).

Lootsmith reads, edits, and writes Borderlands 2 save files: items, weapons,
currency, skills, and progression. It ships as a library (`lootsmith-core`), a
CLI (`lootsmith-cli`), and (later) an `egui` desktop app (`lootsmith-gui`),
with `lootsmith` as a facade crate.

Status: **Phase 0/1 — container unwrap implemented and validated against a real
PC save.** See `../.opencode/plan/LOOTSMITH_PLAN.md` for the full plan.

## Workspace layout

| Crate | Purpose |
|---|---|
| `lootsmith-core` | Container, bit/Huffman codecs, protobuf codec, item codec, game data |
| `lootsmith-cli` | `inspect`, `verify`, `dump` |
| `lootsmith-gui` | `egui` desktop front end (stub) |
| `lootsmith` | Facade re-exporting `lootsmith-core` |

## Save format (verified against upstream `SaveFile.cs`)

```
File (PC / X360 / PS3 / PSVita / Shield)
  = SHA1(20)                    // over everything below
  || u32BE(inner_len)
  || compress(Inner)            // LZO (PC, X360) or Zlib (PS3, PSVita, Shield)

Inner (compressed platforms; the 4-byte prefix is present only here)
  = u32BE(inner_size = len(Inner) - 4)
  || "WSG"
  || u32(version = 2)           // platform endian
  || u32(crc32(proto_bytes))    // platform endian
  || i32(proto_size)            // platform endian
  || Huffman(proto_bytes)       // Gibbed.Gearbox.Common.Huffman

File (Switch, compression = None)
  = "WSG" || u32 version || u32 crc32 || i32 size || Huffman(proto_bytes)
```

When `len(Inner) > 0x40000`, the compressed payload is a block table:
`u32BE(block_count)` then `block_count x (u32BE compressed, u32BE uncompressed)`
then the concatenated blocks.

The outer compressed bytes are **not** byte-identical across compressors, so
Lootsmith gates on the unwrapped `Inner` buffer and the protobuf payload, not
on the whole file.

## Using the CLI

```sh
cargo run -p lootsmith-cli -- inspect /path/to/Save0001.sav
cargo run -p lootsmith-cli -- verify  /path/to/Save0001.sav
cargo run -p lootsmith-cli -- dump    /path/to/Save0001.sav
cargo run -p lootsmith-cli -- items   /path/to/Save0001.sav --dumps "/path/Asset Library Manager.json"
```

Edit and write (in place with a `.bak` backup, or with `--out`):

```sh
cargo run -p lootsmith-cli -- edit /path/to/Save0001.sav \
    --level 61 --currency 1234567 --golden-keys 42 --out /path/to/edited.sav
```

Every command defaults to the PC platform (`--platform pc`); other platforms are
accepted but only PC is tested today. `--dumps` (or `LOOTSMITH_ASSET_LIBRARY`)
points at the Gibbed `Asset Library Manager.json` so asset indices resolve to
names; without it the CLI prints raw indices.

Edit a weapon part by name:

```sh
cargo run -p lootsmith-cli -- edit /path/to/Save0001.sav \
    --item 6 --part elemental --set "GD_Weap_Pistol.elemental.Pistol_Elemental_Fire" \
    --dumps "/path/Asset Library Manager.json" --out /path/to/edited.sav
```

## GUI

```sh
cargo run -p lootsmith-gui
# or open a specific save:
cargo run -p lootsmith-gui -- --open "/path/to/Save0001.sav" \
    --dumps "/path/Asset Library Manager.json"
```

A native window: **Browse…** for the save and the Asset Library, edit the common
scalars, see the item list, and **Save** (in place, creating a `.bak`) or
**Save As**. The Asset Library path enables item names.

### Desktop launcher (interim)

`scripts/lootsmith-gui` builds the release binary on first run and launches it.
A menu entry is installed at `~/.local/share/applications/lootsmith.desktop`
(with an icon), so Lootsmith can be opened from the application menu; opening a
`.sav` with it passes the file via `%f`.

### Driving it for debugging

`scripts/gui-drive.sh` launches the GUI on X11 (Xwayland), finds its window, and
screenshots it, so it can be driven with `xdotool`:

```sh
scripts/gui-drive.sh --open tests/fixtures/pc_Save0001.sav --dumps "$DUMPS"
# -> pid=… window=… shot=/tmp/opencode/lootsmith-shot.png
```

A headless self-test (load → edit → save → reload → verify, no window):

```sh
cargo run -p lootsmith-gui -- --selftest --open tests/fixtures/pc_Save0001.sav
```

Logs go to `~/.cache/lootsmith/gui.log`.

## Tests

A real save is expected at `tests/fixtures/pc_Save0001.sav` (git-ignored). With
it present:

```sh
cargo test -p lootsmith-core
```

validates SHA1, CRC32, LZO decompression, and that the Huffman re-encode
reproduces the original inner buffer byte-for-byte.

## Licensing

Project code is zlib, inherited from upstream. Game data is **not** bundled and
must be supplied by the user; no Gearbox assets are redistributed.
