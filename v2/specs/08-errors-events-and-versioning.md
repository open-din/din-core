# Errors, Events, Compatibility, and Non-Goals

## 1. Error philosophy

The core should prefer structured diagnostics over opaque failures.

### Recommended error layers

- parse error
- validation rejection
- query misuse
- runtime misuse
- unsupported feature
- profile capability mismatch
- bridge mapping error

## 2. Event philosophy

Runtime events should be explicit and host-observable.

Recommended event families:

- transport started/stopped/paused/resumed
- BPM changed
- position changed
- loop entered/completed
- parameter changed
- sequencer triggered/stopped/retriggered
- bridge message accepted/rejected
- runtime warning or fault

## 3. Compatibility rules

`din-core` must follow DinDocument v1 compatibility boundaries:

- `version` changes are breaking at the document container level
- optional profiles are additive
- unsupported required extensions are fatal
- unknown non-required fields are ignored

## 4. Build compatibility

The same semantic behavior should hold across:

- native Rust
- wasm main-thread
- wasm worker

Minor differences in scheduling granularity are acceptable, but validation and query semantics must remain equivalent.

## 5. Security boundaries

The core must treat documents and execution descriptors as untrusted input.

The core must not assume:

- that document-declared executable artifacts are safe
- that host bindings imply permission
- that platform devices are available

## 6. Explicit non-goals

This package does not define:

- a binary document container
- device discovery APIs
- browser UI widgets
- a DSP graph serializer
- a specific audio rendering backend
- a universal OSC transport implementation
- a platform scheduler implementation
