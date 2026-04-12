# AGENTS — din-core

## LOAD ORDER

1. `AGENTS.md`
2. `project/ROUTE_CARD.json` (includes `narrative_before_code` + `narrative_policy`)
3. **Narrative layer only** — `project/SUMMARY.md`, `project/USERFLOW.md`, `project/TEST_MATRIX.md` (no other broad doc tree before code)
4. One matching file in `project/skills/`
5. The exact crate module (and `docs_v2/<slug>.md` only when the task cites that slug)
6. The exact regression test

## ROUTE HERE WHEN

- The request changes DinDocument parsing, validation, handles, graph views, runtime session, WASM bindings, or worker message contracts.
- The request changes canonical DinDocument fixtures or Rust round-trip behavior under `fixtures/din-document-v1/`.

## ROUTE AWAY WHEN

- Public patch schema, react-din package exports, or docs/components -> `react-din`
- Editor, MCP, launcher, or shell work -> `din-studio`
- Workspace routing or automation -> `din-agents`

## ENTRY POINTS

- `project/ROUTE_CARD.json`
- `crates/din-document/src/lib.rs`
- `crates/din-core/src/lib.rs` / `crates/din-core/src/runtime.rs`
- `crates/din-wasm/src/din_document_wasm.rs`

## SKILL MAP

- DinDocument v2 / `v2/specs` tasks -> `project/skills/v2-refactor-task/SKILL.md` (primary workflow)
- FFI or WASM work -> `project/skills/multi-target-wrapper/SKILL.md`
- Rust gates -> `project/skills/rust-quality-gates/SKILL.md`
- Legacy patch-schema / registry skills -> `project/skills/patch-contract-change/SKILL.md`, `project/skills/registry-parity/SKILL.md` (archived; do not use for new work unless explicitly restoring V1 artifacts)

## HARD RULES

- `react-din` owns the published patch JSON schema; this repo’s active contract is **DinDocument v2** (`din-document`, `din-wasm`).
- Keep WASM wrappers thin; no duplicate validation logic outside `din-document`.
- Open `fixtures/din-document-v1/` for DinDocument examples; legacy `fixtures/canonical_patch.json` is not part of the v2 CI surface.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
