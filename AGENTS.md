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
4. `cargo doc --workspace --no-deps` or `./scripts/generate-docs.sh` — must succeed before merge when public Rust API, crate roots, or documented modules change; HTML under `target/doc/`, index stub under `docs/generated/` (gitignored).

Schema and fixtures that mirror the public patch contract: `schemas/patch.schema.json`, `fixtures/canonical_patch.json`.

## Documentation Strategy

- Prefer `README.md`, `docs/**` (when present), and local `target/doc/**` after `./scripts/generate-docs.sh` over scanning raw sources for API shape.
- Load generated HTML rustdoc on demand; it is not default agent context.

## Documentation Rules

- Crate roots and public helpers should carry `//!` / `///` docs; large schema-mirror types document linkage to `schemas/patch.schema.json` instead of duplicating field lists.
- After changing public Rust API, run `./scripts/generate-docs.sh` and fix `missing_docs` warnings when practical.

## Documentation Access Order (CRITICAL)

Always follow this sequence when gathering context. Do not skip steps.

1. This `AGENTS.md` — ownership, rules, quality gates
2. `README.md` and `docs/Architecture.md` — hand-written index; use workspace `docs/README.md` when routing the whole stack
3. Workspace summary `../docs/summaries/din-core-api.md` (when using the `open-din` container) — compressed API overview
4. `target/doc/` after `./scripts/generate-docs.sh` — reference only, narrow to the crate/page needed
5. Source under `crates/` — last resort

## Context Budget Rules

- Load at most two documentation files per step; close or stop using them before opening more
- Load at most one repository’s context unless the task is explicitly cross-repo
- Prefer summaries over rustdoc HTML dumps; prefer rustdoc over reading entire crate sources
- Never bulk-load generated HTML — open only the specific pages needed
- Minimize total loaded context at all times

## Code Reading Policy

- Do **not** read source files when documentation answers the question
- Exhaust summaries and targeted rustdoc before opening `crates/`
- When source reading is required, scope to the exact module — do not scan entire directories

## Documentation Ownership

- This repository owns `docs/`, this `AGENTS.md`, `target/doc/`, and `docs/generated/` output from the doc script
- Workspace summaries (`open-din/docs/summaries/`) must stay consistent when public Rust API or crate boundaries change
- A contract or registry change is incomplete until schema, fixtures, and the matching summary are updated when the surface changes

## Documentation Freshness

- Regenerate docs after public API changes (`./scripts/generate-docs.sh` or `cargo doc --workspace --no-deps`)
- Treat `target/doc/` and generated stubs as ephemeral local artifacts
- After regeneration, decide whether `../docs/summaries/din-core-api.md` needs an update
- Do not cite outdated documentation as authoritative
