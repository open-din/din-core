# SKILL: patch-contract-change

## REPO

`din-core`

## WHEN TO USE

- Patch validation, migration, or round-trip behavior changes
- Public contract parity with `react-din` is in scope

## STEPS

1. Read `project/ROUTE_CARD.json` and the matching `din-patch` module.
2. Update `crates/din-patch` or `crates/din-core` first.
3. Keep `schemas/patch.schema.json` and `fixtures/canonical_patch.json` aligned.
4. Escalate to `react-din` for published schema or persisted ID changes.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
