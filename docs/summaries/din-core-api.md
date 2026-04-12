# din-core workspace — Rust API summary

Generated from crate-level and public-item **rustdoc** in this repository. Authoritative behavior: source under `crates/` and `docs_v2/`. Normative JSON: `open-din/v2` (workspace sibling).

---

## Crate `din_document`

Path: `crates/din-document/src/lib.rs`

**Purpose:** DinDocument v1 core profile — typed model, parse, semantic validation, `DocumentHandle`, and scene route graph utilities.

### Re-exports

| Symbol | Module | Summary |
| --- | --- | --- |
| `DinDocument`, model types | `model` | Serde model aligned with `open-din/din-document` |
| `parse_document_json_str` | `parse` | Parse UTF-8 JSON text → `DinDocument` or `ParseError` |
| `parse_document_json_value` | `parse` | Parse `serde_json::Value` → `DinDocument` or `ParseError` |
| `ParseError` | `parse` | Parse failure with `message: String` |
| `validate_document` | `validate` | Semantic validation → `ValidationReport` |
| `ValidationReport`, `ValidationIssue`, `IssueCode` | `report` | Diagnostics and stable issue codes |
| `DocumentHandle`, `DocumentHandleBuildError`, `SceneGraphView` | `handle` | Read-only indexed access after accepted validation |
| `SceneRouteGraph`, `build_scene_route_graph`, `directed_graph_has_cycle`, `route_endpoint_key`, `topological_order` | `graph` | Route graph views and cycle/DAG helpers |

### `parse`

- `parse_document_json_str(text: &str) -> Result<DinDocument, ParseError>` — deserialize JSON text.
- `parse_document_json_value(value: Value) -> Result<DinDocument, ParseError>` — deserialize from a value.

### `validate`

- `validate_document(doc: &DinDocument) -> ValidationReport` — semantic rules (format/version, default scene, routes, profiles, etc.); see module rustdoc in `validate.rs`.

### `handle` (selection)

- `DocumentHandle::try_new(doc, report)` — succeeds only if `report.accepted`.
- Query methods: scenes, `scene(id)`, `default_scene()`, `graph(scene_id)`, collection lookups — see `handle.rs` rustdoc.

### `graph`

- `build_scene_route_graph(scene, …)` — normalized nodes/edges for routing.
- `directed_graph_has_cycle`, `topological_order` — DAG analysis.

---

## Crate `din_core`

Path: `crates/din-core/src/lib.rs`

**Purpose:** Re-export `din_document::*` and expose the **runtime** module plus `DIN_CORE_VERSION`.

### Re-exports

- `pub use din_document::*;` — full document stack in one dependency when needed.

### `runtime` (`crates/din-core/src/runtime.rs`)

| Item | Summary |
| --- | --- |
| `RuntimeSession` | `Arc<DocumentHandle>` + selected `scene_id`; controllers for transport, sequencer, bridge |
| `RuntimeSession::new(handle, scene_id)` | `Err(UnknownScene)` if scene missing |
| `RuntimeSessionError::UnknownScene` | Unknown scene id |
| `TransportController` | `playing`, `paused`, `bpm`, `seek_beats`; `apply(TransportCommand)`; `tick()` |
| `TransportCommand` | `Start`, `Stop`, `Pause`, `Resume`, `Seek(f64)` |
| `SequencerController` | `trigger`, `retrigger`, `stop`, `generation`, `tick` |
| `BridgeController` | Placeholder |

### Constants

- `DIN_CORE_VERSION` — `env!("CARGO_PKG_VERSION")` string for bindings.

---

## Crate `din_wasm`

Path: `crates/din-wasm/src/lib.rs` → `din_document_wasm.rs`

**Purpose:** `wasm-bindgen` exports for validation JSON, opaque handles, worker dispatch, version string. Not a second validation implementation — delegates to `din-document` / `din-core`.

### Public Rust re-exports (for tests / embedding)

- `din_document_validate_json`, `din_document_validate_json_impl`
- `WasmDinDocumentHandle`
- `worker_dispatch_message_json`, `worker_dispatch_message_json_impl`
- `din_core_version`

See **JavaScript names** in `crates/din-wasm/pkg/docs/api.md`.

---

## Regenerating HTML rustdoc

```bash
cargo doc --workspace --no-deps --open
```

This Markdown file is a **stable summary** for agents and cross-repo links; HTML docs remain the full rustdoc source.
