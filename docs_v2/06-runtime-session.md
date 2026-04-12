# Runtime session (DinDocument v1)

## Behavior

- [`RuntimeSession`](../../crates/din-core/src/document_v1.rs) holds an `Arc<DocumentHandle>` and a selected scene id; construction fails with [`RuntimeSessionError::UnknownScene`](../../crates/din-core/src/document_v1.rs) when the id is missing.
- Document data stays immutable behind the handle; mutable state lives in controllers on the session.

## Tests

- `crates/din-core/src/document_v1.rs` (unknown scene rejection).
