# Native vs WASM validation parity

## Check

- Integration test [`wasm_din_document_validate_matches_native`](../../crates/din-wasm/tests/wasm.rs) compares [`validate_document`](../../crates/din-document/src/validate.rs) with [`din_document_validate_json_impl`](../../crates/din-wasm/src/din_document_wasm.rs) on the same UTF-8 bytes (`minimal.din.json`).
