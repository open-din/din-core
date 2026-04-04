# Skill: patch-contract-change

## Triggers

- Add or change a patch node kind, interface field, or migration behavior.
- Prompts about round-trip preservation or `PatchDocument` compatibility with `react-din`.

## Workflow

1. Read `project/TEST_MATRIX.md` scenarios `F01-*` and `project/features/01_patch_contract.feature.md`.
2. Update Rust logic in `din-patch` / `din-core` first; keep `din-ffi` and `din-wasm` thin adapters.
3. Align with `schemas/patch.schema.json` and extend `fixtures/canonical_patch.json` when the public shape changes (coordinate a sibling `react-din` schema PR if it is the published copy).
4. Add or update tests that prove round-trip and interface parity.

## Checks

- `cargo test -p din-patch`
- `cargo test --workspace`
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`

## Expected outputs

- Patch behavior, fixtures, and docs references match; no silent drift from the node registry.
