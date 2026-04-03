# AGENTS

## Product rule

Keep `react-din` patch compatibility as the top external contract for this repository.

## Naming rule

- Persist exact patch node IDs such as `osc`, `stepSequencer`, and `midiCC`.
- Use PascalCase Rust variants and structs derived from those IDs.
- Document every non-trivial alias in the node registry.

## Implementation rule

- Prefer adding behavior behind `din-patch` and `din-core` before widening FFI or WASM.
- Keep one authoritative node registry for compiler, docs, and tests.
- Treat round-trip patch preservation and interface naming parity as release gates.

## Testing rule

- Every supported patch node type must appear in registry parity tests.
- Patch migration and round-trip tests must preserve interface metadata and asset paths.
- FFI and WASM wrappers should stay thin and reuse Rust-native logic.
