# Worker message dispatch (JSON)

## Behavior

- [`workerDispatchMessageJson`](../../crates/din-wasm/src/din_document_wasm.rs) accepts JSON `{ "family": string, "payload": object }`.
- Families: `document/open` (requires `payload.json` string), `runtime/create`, `transport/tick` — stub success envelopes for the latter two.

## Tests

- `worker_dispatch_message_json_impl` in `crates/din-wasm/tests/wasm.rs`.
