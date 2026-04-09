# AGENTS

Canonical contract for Codex, Claude, Cursor, and other agents. Product narratives live in `project/SUMMARY.md`, `project/USERFLOW.md`, and `project/TEST_MATRIX.md`—cite them; do not duplicate them here. Portable workflows: `project/skills/*/SKILL.md`.

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

## Quality gates (pre-merge)

Run from the repository root:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. (Recommended for API changes) `cargo doc --workspace --no-deps` or `./scripts/generate-docs.sh` — confirms rustdoc builds; HTML lands in `target/doc/`, with a short index refreshed under `docs/generated/` (gitignored).

Schema and fixtures that mirror the public patch contract: `schemas/patch.schema.json`, `fixtures/canonical_patch.json`.

## Documentation Strategy

- Prefer `README.md`, `docs/**` (when present), and local `target/doc/**` after `./scripts/generate-docs.sh` over scanning raw sources for API shape.
- Load generated HTML rustdoc on demand; it is not default agent context.

## Documentation Rules

- Crate roots and public helpers should carry `//!` / `///` docs; large schema-mirror types document linkage to `schemas/patch.schema.json` instead of duplicating field lists.
- After changing public Rust API, run `./scripts/generate-docs.sh` and fix `missing_docs` warnings when practical.
