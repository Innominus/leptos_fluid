# wasm_strip technical.md

This document explains the standalone `wasm_strip` utility crate under `tools/wasm_strip`.

## Purpose

`wasm_strip` removes selected WebAssembly custom sections from an input `.wasm` file to reduce output size.

Current stripped section names:

- `__wasm_bindgen_unstable`
- `name`
- `producers`

## Execution flow

`src/main.rs`:

1. parse CLI args: `wasm_strip <input.wasm> [output.wasm]`
2. read full input bytes
3. parse wasm section stream
4. copy all sections except matching custom sections
5. write output bytes

If no output path is provided, it writes `<input>.stripped.wasm`.

## Parsing model

The implementation is intentionally minimal and stream-based:

- validates wasm magic header (`\0asm`) and version bytes exist
- iterates sections from byte offset 8
- reads section size via unsigned LEB128 (`read_leb_u32`)
- for custom sections (`id == 0`), parses section name and applies strip filter

All kept sections are copied byte-for-byte from original payload.

## Why this approach

- no dependency on external wasm parser crates
- deterministic behavior and small binary
- preserves unknown standard/custom sections unless explicitly stripped

## Failure handling

The utility exits non-zero on:

- missing input argument
- file read/write errors
- malformed wasm structure (invalid section boundaries/LEB decoding)

Parsing helpers return `Option` and let caller convert into explicit CLI errors.

## Contributor notes

- If adding new strip targets, update the static list in `main`.
- Keep parsing strict around bounds checks (`checked_add`, payload end guards).
- Do not rewrite section payloads; this tool is a selective copier, not a wasm re-encoder.
