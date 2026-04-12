# WASM: DinDocument validation and handles

## Exports

- [`dinDocumentValidateJson`](../../crates/din-wasm/src/din_document_wasm.rs) — returns `{ accepted, issues[] }` with stable snake_case `code` strings.
- [`WasmDinDocumentHandle`](../../crates/din-wasm/src/din_document_wasm.rs) — constructed only for accepted documents; [`dispose`](../../crates/din-wasm/src/din_document_wasm.rs) drops the Rust handle.
- [`dinCoreVersion`](../../crates/din-wasm/src/din_document_wasm.rs) — forwards [`din_core::DIN_CORE_VERSION`](../../crates/din-core/src/lib.rs).

Legacy patch migration, graph export, audio render, and node catalog helpers are **not** exported from this crate.

## Tests

- `din_document_validate_json_impl` used from integration tests (`crates/din-wasm/tests/wasm.rs`).
