# din-core

`din-core` is a Rust-first native library workspace that mirrors the canonical `react-din` patch contract and prepares the runtime for reuse across native, C ABI, and WebAssembly targets.

## Outcome

`din-core` helps host applications import, validate, migrate, inspect, compile, and run `react-din`-compatible audio patch graphs from a single Rust codebase.

## Workspace

- `crates/din-patch`: canonical patch types, naming, validation, migration, and graph round-trip helpers.
- `crates/din-core`: graph model, node registry, compiler, runtime engine, notes helpers, and data helpers.
- `crates/din-ffi`: C ABI wrapper over patch and engine APIs.
- `crates/din-wasm`: WebAssembly-facing patch/model helpers.

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
