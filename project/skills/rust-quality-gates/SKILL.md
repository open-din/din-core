# SKILL: rust-quality-gates

## REPO

`din-core`

## WHEN TO USE

- You need the final validation pass for a Rust-side change
- The request touches any runtime, registry, or wrapper surface

## STEPS

1. Read `project/ROUTE_CARD.json` for the required commands and boundaries.
2. Verify ownership stayed inside `din-core` unless a shared contract changed.
3. Run the Rust validation commands in repo order.
4. Regenerate docs only if public Rust-facing surfaces changed.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
