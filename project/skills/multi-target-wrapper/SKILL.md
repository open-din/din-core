# Skill: multi-target-wrapper

## Triggers

- Changes under `din-ffi` or `din-wasm`.
- Exposing new engine or patch helpers across C ABI or WASM.

## Workflow

1. Read `project/TEST_MATRIX.md` scenarios `F03-*` and `project/features/03_multi_target_exports.feature.md`.
2. Implement new behavior in `din-core` / `din-patch` first; keep wrappers as thin forwarding layers.
3. Add integration tests in the FFI and WASM crates that reuse Rust-native structs and errors.
4. Avoid widening exported surface area until native APIs are stable.

## Checks

- `cargo test -p din-ffi`
- `cargo test -p din-wasm`
- `cargo test --workspace`
- `./scripts/generate-docs.sh` after new `pub` exports on FFI/WASM surfaces

## Expected outputs

- Cross-target exports stay consistent; wrappers do not reimplement business logic.
