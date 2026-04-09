# SKILL: registry-parity

## REPO

`din-core`

## WHEN TO USE

- Node registry, node IDs, or interface metadata changes
- A request mentions registry parity with public contracts or studio metadata

## STEPS

1. Read the summary files, repo manifest, and `fixtures/canonical_patch.json`.
2. Update registry logic in Rust first.
3. Preserve persisted IDs and interface naming unless an explicit migration exists.
4. Escalate only if shared IDs or public schema expectations change.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
