# SKILL: multi-target-wrapper

## REPO

`din-core`

## WHEN TO USE

- FFI or WASM bindings change
- Native Rust behavior already moved and wrappers must follow

## STEPS

1. Read the repo summary, API summary, and repo manifest.
2. Confirm the native Rust behavior exists in `din-patch` or `din-core`.
3. Update `din-ffi` or `din-wasm` as thin adapters only.
4. Avoid duplicating validation, migration, or runtime logic in wrappers.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
