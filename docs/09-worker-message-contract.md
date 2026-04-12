# Worker message dispatch (JSON)

## Behavior

- [`workerDispatchMessageJson`](../../crates/din-wasm/src/din_document_wasm.rs) accepts JSON `{ "family": string, "payload": object }`.
- **`document/open`** — requires `payload.json` (UTF-8 DinDocument JSON). Returns `{ ok, accepted, issueCodes }` from semantic validation.
- **`runtime/create`** — requires `payload.json`. Optional `payload.sceneId`; when omitted, the document’s `defaultSceneId` is used. On success stores a [`RuntimeSession`](../../crates/din-core/src/runtime.rs) and returns `{ ok, accepted, sceneId, transport, sequencer, bridge }`. On validation failure returns `{ ok: false, accepted: false, issueCodes }`. On unknown scene returns `{ ok: false, accepted: false, error: { code: "UnknownScene", message } }`.
- **`transport/tick`** — advances the stored session’s [`TransportController::tick`](../../crates/din-core/src/runtime.rs); returns `{ ok, transport, sequencer }` or an error string if no session exists (call `runtime/create` first).

## Tests

- `crates/din-wasm/tests/wasm.rs`: `wasm_worker_document_open_round_trip`, `wasm_worker_runtime_create_and_transport_tick`, `wasm_worker_runtime_create_unknown_scene_returns_error_envelope`.
