# Skill: rust-quality-gates

## Triggers

- Any PR before merge, or CI failures on formatting, clippy, or tests.

## Workflow

1. Run `cargo fmt --all` locally if formatting fails; re-run with `--check`.
2. Fix `cargo clippy --workspace --all-targets -- -D warnings` issues without silencing unless justified in review.
3. Run `cargo test --workspace`; if a crate is intentionally scoped, document the exception in the PR.

## Checks

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./scripts/generate-docs.sh` when rustdoc-facing `pub` items changed (`docs/generated/` + `target/doc/` are gitignored)

## Expected outputs

- CI-green Rust workspace before merge.
