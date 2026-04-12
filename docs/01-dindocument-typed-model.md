# DinDocument typed model

**Task:** `v2/features/01-dindocument-typed-model.feature`  
**Crate:** [`crates/din-document`](../crates/din-document)

## Purpose

Provides a serde-aligned Rust model for **DinDocument v1 core** interchange so hosts can deserialize JSON without file I/O in the library.

## Public surface

- [`DinDocument`](../crates/din-document/src/model.rs) — root document type.
- Supporting types: `Scene`, `Route`, `RouteEndpoint`, `Collections`, `Transport`, `Timeline`, ports, DSP definitions, etc.
- Constants: `DOCUMENT_FORMAT`, `DOCUMENT_VERSION`.
- Helpers: `DinDocument::scene_by_id`, `DinDocument::scene_ids`, `DinDocument::summary`.

## Tests (TDD)

- Integration: [`crates/din-document/tests/parse_corpus.rs`](../crates/din-document/tests/parse_corpus.rs) — `minimal.din.json`, `orchestrated-scene.din.json`.

## Fixtures

- [`fixtures/din-document-v1/`](../fixtures/din-document-v1) — copies of `open-din/v2/examples` for CI.
