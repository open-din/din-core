# Parse pipeline

**Task:** `v2/features/02-parse-pipeline.feature`  
**Crate:** [`crates/din-document`](../crates/din-document)

## Purpose

Accept DinDocument JSON already obtained by the host:

- UTF-8 text → [`parse_document_json_str`](../crates/din-document/src/parse.rs)
- `serde_json::Value` → [`parse_document_json_value`](../crates/din-document/src/parse.rs)

## Errors

`ParseError` carries a single `message` string from the underlying `serde_json` failure (malformed JSON or schema shape mismatch).

## Tests

- Unit: malformed JSON in [`parse::tests`](../crates/din-document/src/parse.rs).
- Integration: round-trip text vs value in `parse_corpus::minimal_round_trip_parse`.
