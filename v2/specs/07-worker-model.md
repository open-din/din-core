# Web Worker Execution Model

## 1. Goal

`din-core-wasm` must support deployment inside a Web Worker to reduce main-thread load.

## 2. Worker deployment model

Recommended model:

- the browser main thread owns UI and device integration
- the worker owns `WasmDocumentHandle` and `WasmRuntimeSession`
- the main thread sends commands
- the worker sends back results, state updates, and emitted events

## 3. Responsibilities by side

### Main thread

- file picking / fetching
- UI rendering
- Web MIDI / user gesture integration if needed
- AudioContext lifecycle if not worker-owned
- forwarding external events to the worker

### Worker

- parsing and validation
- document indexing
- graph queries
- transport stepping
- sequencer updates
- bridge conversion logic
- event batching

## 4. Message categories

Recommended message families:

- `document/open`
- `document/validate`
- `document/query`
- `runtime/create`
- `runtime/dispose`
- `transport/command`
- `transport/tick`
- `sequencer/command`
- `bridge/input`
- `events/drain`

## 5. Batching rules

To reduce `postMessage` overhead:

- transport events may be batched
- parameter changes may be batched
- graph snapshots should be requested on demand, not streamed continuously
- validation diagnostics should be returned once per load/validate cycle

## 6. Timing considerations

A worker-safe runtime must not require synchronous access to DOM or main-thread browser objects.

The worker receives:

- high-level commands
- external clock events
- parameter updates
- MIDI/OSC messages normalized by the host

## 7. Worker-safe bridge model

Bridge logic inside the worker should consume normalized message objects rather than raw browser device handles.

Examples:

- normalized MIDI event object
- normalized OSC event object
- normalized clock event object

## 8. Failure handling

The worker must surface structured failures for:

- invalid document input
- invalid runtime commands
- unknown scene or sequencer ids
- invalid bridge mappings
- unsupported operations in the current build
