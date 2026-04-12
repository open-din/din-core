# User flow

End-to-end flow for **DinDocument** handling in this repository (v2 feature slice 01–12). Steps map to `project/features/` and `docs_v2/`.

1. **Load JSON** — A host obtains UTF-8 DinDocument JSON (file, network, or in-memory string) matching `open-din/din-document` format and version `1`.

2. **Parse** — `parse_document_json_str` or `parse_document_json_value` (`din-document`) produces a typed `DinDocument` or a structured `ParseError`.

3. **Validate** — `validate_document` returns a `ValidationReport` with `accepted` and `issues` (`IssueCode`, message, optional path). Gates include format/version, default scene resolution, route cycles, and optional **execution** / **host-binding** profiles when declared.

4. **Handle** — For accepted reports, `DocumentHandle::try_new` exposes read-only queries: scenes, default scene, per-scene graph view, buffers, etc.

5. **Graph** — For routing, `build_scene_route_graph` and helpers provide DAG views; cyclic routes yield validation or graph diagnostics.

6. **Runtime session** — Optionally, `RuntimeSession::new(Arc<DocumentHandle>, scene_id)` binds mutable controllers to one scene; unknown scene ids fail with `RuntimeSessionError::UnknownScene`.

7. **Transport / sequencer / bridge** — `TransportController` (commands + `tick`), `SequencerController` (trigger / retrigger / stop), `BridgeController` (placeholder) live on the session.

8. **WASM** — In the browser, `dinDocumentValidateJson` mirrors validation output as plain objects; `WasmDinDocumentHandle` holds an accepted handle until `dispose`.

9. **Worker** — `workerDispatchMessageJson` accepts `{ family, payload }` for `document/open`, `runtime/create`, and `transport/tick`, returning structured JSON envelopes (see `docs_v2/09-worker-message-contract.md`).

10. **Parity** — Native `validate_document` and WASM validation helpers are compared on shared fixture bytes in tests (`crates/din-wasm/tests/wasm.rs`).

11. **Quality gates** — CI runs `cargo fmt`, `clippy -D warnings`, and `cargo test --workspace` (`project/ROUTE_CARD.json`).
