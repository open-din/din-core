# AGENTS — din-core

## LOAD ORDER

1. `AGENTS.md`
2. `project/ROUTE_CARD.json`
3. One matching file in `project/skills/`
4. The exact crate module
5. The exact regression test

## ROUTE HERE WHEN

- The request changes runtime, compiler, registry, migration, validation, FFI, or WASM behavior.
- The request changes canonical fixtures or Rust round-trip behavior.

## ROUTE AWAY WHEN

- Public API, exports, docs/components, or published schema -> `react-din`
- Editor, MCP, launcher, or shell work -> `din-studio`
- Workspace routing or automation -> `din-agents`

## ENTRY POINTS

- `project/ROUTE_CARD.json`
- `crates/din-core/src/engine.rs`
- `crates/din-core/src/registry.rs`
- `crates/din-patch/src/document.rs`

## SKILL MAP

- Patch contract change -> `project/skills/patch-contract-change/SKILL.md`
- Registry or node ID change -> `project/skills/registry-parity/SKILL.md`
- FFI or WASM work -> `project/skills/multi-target-wrapper/SKILL.md`
- Rust gates -> `project/skills/rust-quality-gates/SKILL.md`

## HARD RULES

- `react-din` owns the published schema; this repo owns runtime semantics and registry authority.
- Persisted node IDs stay stable.
- Open `fixtures/canonical_patch.json` only when patch parity or round-trip behavior is in scope.
- Keep FFI and WASM thin.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
