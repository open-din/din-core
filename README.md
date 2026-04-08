# din-core

`din-core` is a Rust-first native library workspace that mirrors the canonical `react-din` patch contract and prepares the runtime for reuse across native, C ABI, and WebAssembly targets.

## Outcome

`din-core` helps host applications import, validate, migrate, inspect, compile, and run `react-din`-compatible audio patch graphs from a single Rust codebase.

## Workspace

- `crates/din-patch`: canonical patch types, naming, validation, migration, and graph round-trip helpers.
- `crates/din-core`: graph model, node registry, compiler, runtime engine, notes helpers, and data helpers.
- `crates/din-ffi`: C ABI wrapper over patch and engine APIs.
- `crates/din-wasm`: WebAssembly-facing patch/model helpers.

## Quickstart (Rust / `din-patch`)

Typical flow for host tooling: parse interchange JSON, validate invariants, then (optionally) serialize again.

```rust
use din_patch::{parse_patch_document, validate_patch_document, PatchError};

fn load() -> Result<(), PatchError> {
    let json = std::fs::read_to_string("fixtures/canonical_patch.json").map_err(|e| {
        PatchError::Invalid(format!("read fixture: {e}"))
    })?;
    let patch = parse_patch_document(&json)?;
    validate_patch_document(&patch)?;
    Ok(())
}
```

- Authoritative JSON shape: `schemas/patch.schema.json`.
- Fixture used in tests and docs: `fixtures/canonical_patch.json`.
- Node `type` fields must match the workspace registry (for example `osc`, `stepSequencer`, `midiCC`); see `AGENTS.md` and `project/TEST_MATRIX.md`.

Runnable example (validate + serde round-trip check):

```bash
cargo run -p din-patch --example parse_canonical_patch
```

## Current v1 scope

- Keep `PatchDocument v1` from `react-din` as the public exchange format.
- Keep patch node type IDs as the canonical persisted identifiers.
- Expose Rust-native APIs plus a stable C ABI and a model-focused WASM package.
- Exclude app shells and high-level synth helpers such as `Synth`, `MonoSynth`, `FMSynth`, `AMSynth`, `NoiseSynth`, `DrumSynth`, and `PolyVoice`.

## Development

```bash
cargo test
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Runtime status

The runtime engine included here is intentionally conservative in v1: it compiles and classifies the full `react-din` patch surface, exposes control and MIDI entry points, and renders a deterministic audio buffer with lightweight processor behavior for the implemented core node set. The registry and compiler are designed so richer DSP can be added without breaking the public patch contract.
