# AGENTS — din-core

## LOAD ORDER

1. `AGENTS.md`
2. `project/SUMMARY.md`
3. `../docs/summaries/din-core-api.md`
4. `project/REPO_MANIFEST.json`
5. One matching file in `project/skills/`

## ROUTE HERE WHEN

- The request changes runtime, compiler, registry, migration, validation, FFI, or WASM behavior.
- The request changes canonical fixtures or Rust round-trip behavior.

## ROUTE AWAY WHEN

- Public API, exports, docs/components, or published schema -> `react-din`
- Editor, MCP, launcher, or shell work -> `din-studio`
- Workspace routing or automation -> `din-agents`

## ENTRY POINTS

- `crates/din-patch`
- `crates/din-core`
- `schemas/patch.schema.json`
- `fixtures/canonical_patch.json`

## SKILL MAP

- Patch contract change -> `project/skills/patch-contract-change/SKILL.md`
- Registry or node ID change -> `project/skills/registry-parity/SKILL.md`
- FFI or WASM work -> `project/skills/multi-target-wrapper/SKILL.md`
- Rust gates -> `project/skills/rust-quality-gates/SKILL.md`

## HARD RULES

- `react-din` owns the published schema; this repo owns runtime semantics and registry authority.
- Persisted node IDs stay stable.
- Keep FFI and WASM thin.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
