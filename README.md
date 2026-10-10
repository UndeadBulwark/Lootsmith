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
```

Both default to the PC platform (`--platform pc`); other platforms are
accepted but only PC is tested today.

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
