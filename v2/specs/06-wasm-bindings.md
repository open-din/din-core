# WASM Binding Crate Specification

## 1. Crate identity

The WASM-facing crate must be separate from `din-core`.

Recommended name:

- `din-core-wasm`

## 2. Goals

The binding crate must:

- expose JS-friendly document loading APIs
- expose JS-friendly validation reports
- expose query methods for scenes, assets, ports, routes, and graphs
- expose runtime session APIs for transport and sequencers
- support both browser main-thread and Worker use

## 3. API style

Two API styles may coexist:

### Object API

The binding crate exports classes/handles such as:

- `WasmDinCore`
- `WasmDocumentHandle`
- `WasmRuntimeSession`

### Message API

The binding crate exports a worker-friendly dispatcher such as:

- `handle_message(message) -> response`

## 4. JS boundary rules

The WASM boundary should:

- avoid leaking Rust-internal types
- return plain JS values for summaries, diagnostics, and graph snapshots
- use stable ids/handles for long-lived objects
- minimize serialization in runtime hot paths

## 5. Required binding operations

### Document operations

- open document from JSON text
- open document from JSON object
- validate document
- get document summary
- query scenes
- query assets
- query scene inputs/outputs/routes
- get graph snapshot

### Runtime operations

- create runtime from selected scene
- transport start/stop/pause/resume/reset/seek
- get transport state
- set/get parameters
- subscribe or poll emitted events
- trigger sequencers
- submit bridge input messages

## 6. Error mapping

Rust errors must be converted into stable JS-facing error objects containing:

- code
- message
- details if available

## 7. Memory management

The binding layer must define a clear ownership model for:

- document handles
- runtime handles
- event buffers

The host must be able to explicitly dispose long-lived handles.
