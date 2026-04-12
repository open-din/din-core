# Transport, sequencer, and bridge controllers

## Behavior

- [`TransportController`](../../crates/din-core/src/document_v1.rs) applies [`TransportCommand`](../../crates/din-core/src/document_v1.rs) (start/stop/pause/resume/seek) and exposes a scalar-only [`tick`](../../crates/din-core/src/document_v1.rs) for hot paths.
- [`SequencerController`](../../crates/din-core/src/document_v1.rs) provides `trigger` / `retrigger` / `stop` with deterministic generation counting.
- [`BridgeController`](../../crates/din-core/src/document_v1.rs) is a placeholder for future bridge mapping.

## Tests

- Transport tick loop and session wiring: `document_v1` tests in `crates/din-core`.
