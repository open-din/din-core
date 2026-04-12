# din-core v2 documentation

English-only technical notes for the DinDocument v1 refactor tracked under `v2/specs` and `tasks/`.

This repository no longer ships the legacy react-din **patch JSON** stack (`din-patch`), **C FFI** (`din-ffi`), native **DSP audio engines**, or exhaustive **NodeKind** registry parity. The supported surface is **DinDocument v2** only: typed model through WASM/worker helpers and a minimal **RuntimeSession** with transport / sequencer / bridge controllers.

Each page corresponds to a Gherkin task in `tasks/todo` (active) or `v2/features` (completed).

| Doc | Task | Crate |
|-----|------|--------|
| [01-dindocument-typed-model.md](01-dindocument-typed-model.md) | Typed model | `crates/din-document` |
| [02-parse-pipeline.md](02-parse-pipeline.md) | Parse pipeline | `crates/din-document` |
| [03-validation-and-diagnostics.md](03-validation-and-diagnostics.md) | Validation | `crates/din-document` |
| [04-document-handle-query.md](04-document-handle-query.md) | DocumentHandle / query API | `crates/din-document` |
| [05-graph-view.md](05-graph-view.md) | Route graph view | `crates/din-document` |
| [06-runtime-session.md](06-runtime-session.md) | Runtime session skeleton | `crates/din-core` |
| [07-transport-sequencer-bridge.md](07-transport-sequencer-bridge.md) | Transport / sequencer / bridge | `crates/din-core` |
| [08-wasm-bindings.md](08-wasm-bindings.md) | DinDocument WASM | `crates/din-wasm` |
| [09-worker-message-contract.md](09-worker-message-contract.md) | Worker JSON dispatch | `crates/din-wasm` |
| [10-cross-target-parity.md](10-cross-target-parity.md) | Native vs WASM validation parity | `crates/din-wasm`, `din-document` |
| [11-execution-profile.md](11-execution-profile.md) | Execution profile gates | `crates/din-document` |
| [12-host-binding-profile.md](12-host-binding-profile.md) | Host binding profile gates | `crates/din-document` |

Normative JSON: `open-din/v2/schema/din-document.core.schema.json` (workspace sibling). Example corpus: `fixtures/din-document-v1/` (synced from `open-din/v2/examples`).

## Cross-repo v2 references

| Role | Path |
|------|------|
| Normative DinDocument | `open-din/v2/` |
| Core specs | `v2/specs/` (this repo) |
| Studio product specs | `din-studio/v2/specs/` |

## Agent documentation load order (minimize context)

1. `din-core-specs/` — **only** when the task cites legacy dossier sections.
2. This folder — matching task slug when present.
3. `v2/specs/` — **only** files cited by the task.
4. `v2/user-stories/` — **only** `.feature` files the task references.
5. `open-din/v2` — **only** paths cited in the task.
