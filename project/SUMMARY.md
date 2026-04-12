# SUMMARY

With `project/USERFLOW.md` and `project/TEST_MATRIX.md`, this is the **only** repo-wide narrative layer before opening `crates/` (see `project/ROUTE_CARD.json` → `narrative_before_code`).

## PURPOSE

Rust workspace for **DinDocument v1** (core profile): typed JSON model, parse pipeline, semantic validation, read-only `DocumentHandle` and route **graph** views, a minimal **`RuntimeSession`** with transport / sequencer / bridge controllers, and **WebAssembly** bindings plus a **worker** JSON dispatcher.

Normative interchange: `open-din/v2` (schema and examples). Local corpus: `fixtures/din-document-v1/`.

## OWNS

- `din-document`: `DinDocument` serde model, `parse_document_json_str` / `parse_document_json_value`, `validate_document`, `ValidationReport` / `IssueCode`, `DocumentHandle`, scene route graph helpers (`build_scene_route_graph`, cycle detection, topological order).
- `din-core`: thin re-export of `din-document` plus `runtime` (`RuntimeSession`, `TransportController`, `SequencerController`, `BridgeController`) and `DIN_CORE_VERSION` — see `crates/din-core/src/runtime.rs`.
- `din-wasm`: `dinDocumentValidateJson`, `WasmDinDocumentHandle`, `workerDispatchMessageJson`, `dinCoreVersion`; test helpers for native parity.

## DOES NOT OWN

- Legacy **react-din patch** JSON schema authority (see `react-din`).
- **DSP audio** engines, **NodeKind** registries, **C FFI** (removed from this workspace).
- Editor UX, MCP, shell, or workspace automation.

## USE WHEN

- The task changes DinDocument parsing, validation, handles, graph views, runtime session, WASM surface, or worker message contracts—see `project/features/*.feature` (01–12).

## DO NOT USE WHEN

- Published patch package API or React components → `react-din`
- Editor / studio workflows → `din-studio`
- Crew routing or quality-gate orchestration → `din-agents`

## RELATED REPOS

- `react-din`: published patch schema and JS/TS patch helpers (separate from DinDocument v2 in this repo).
- `din-studio`: consumes documents and tooling aligned with `open-din/v2` where integrated.
- `din-agents`: routes tasks using `project/ROUTE_CARD.json` and repo manifests.

## FEATURE INDEX

| ID | Feature file | Primary crate |
| --- | --- | --- |
| 01 | `project/features/01-dindocument-typed-model.feature` | `din-document` |
| 02 | `project/features/02-parse-pipeline.feature` | `din-document` |
| 03 | `project/features/03-validation-and-diagnostics.feature` | `din-document` |
| 04 | `project/features/04-document-handle-query.feature` | `din-document` |
| 05 | `project/features/05-graph-view.feature` | `din-document` |
| 06 | `project/features/06-runtime-session.feature` | `din-core` |
| 07 | `project/features/07-transport-sequencer-bridge.feature` | `din-core` |
| 08 | `project/features/08-wasm-bindings.feature` | `din-wasm` |
| 09 | `project/features/09-worker-message-contract.feature` | `din-wasm` |
| 10 | `project/features/10-cross-target-parity.feature` | `din-document`, `din-wasm` |
| 11 | `project/features/11-execution-profile.feature` | `din-document` |
| 12 | `project/features/12-host-binding-profile.feature` | `din-document` |
