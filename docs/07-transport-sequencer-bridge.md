# Transport, sequencer, and bridge controllers

## Behavior

- [`TransportController`](../../crates/din-core/src/runtime.rs) applies [`TransportCommand`](../../crates/din-core/src/runtime.rs) (start/stop/pause/resume/seek) and exposes a scalar-only [`tick`](../../crates/din-core/src/runtime.rs) for hot paths.
- [`SequencerController`](../../crates/din-core/src/runtime.rs) provides `trigger` / `retrigger` / `stop` with deterministic generation counting.
- [`BridgeController`](../../crates/din-core/src/runtime.rs) is a placeholder for future bridge mapping.

## Tests

- Transport tick loop: `transport_commands_and_tick_no_panic` in `crates/din-core/src/runtime.rs`.
