# Runtime, Transport, and Performance Requirements

## 1. Runtime session

A runtime session is the mutable execution-facing object created from a validated document handle and one selected scene.

A runtime session must expose:

- transport state and control
- sequencer control
- parameter get/set APIs
- event subscription
- bridge mapping entry points

## 2. Transport model

The transport subsystem must support:

- start
- stop
- pause
- resume
- reset
- seek
- BPM read/write
- playing state read/write
- time signature read/write
- swing read/write
- loop state read/write
- external clock input
- transport event callbacks

## 3. Parameter interaction

The runtime must support:

- set one parameter
- set many parameters atomically or batch-wise
- get current parameter state
- subscribe to parameter changes
- inject parameter changes from bridges or automation

## 4. Sequencer runtime

The runtime must support:

- create sequencer instances from scene timeline definitions
- query sequencer state
- trigger sequencer
- retrigger sequencer
- stop sequencer
- manage track playback state
- surface MIDI-track and automation-track behavior

## 5. WebAssembly performance target

Transport behavior must be suitable for WebAssembly deployment.

### Requirements

- no mandatory allocation on every transport tick
- no mandatory JSON serialization in hot paths
- no blocking host callbacks inside the timing-critical path
- no reliance on wall-clock sleeps inside core logic
- support incremental state updates instead of full document re-materialization
- transport state mutation should avoid unnecessary cloning of scene or graph data

## 6. Web performance guidance

For browser deployment:

- use the platform clock as an input, not as an internal source of truth
- allow the host to drive tick/update calls
- separate transport stepping from UI rendering frequency
- keep validation and indexing off the hot path
- prefer precomputed indices and route lookup tables

## 7. Suggested stepping model

Recommended runtime stepping API shape:

- `advance_to(time)`
- `process_clock_event(event)`
- `drain_emitted_events()`

This allows the host to:

- drive the engine from `requestAnimationFrame`
- drive the engine from a worker timer
- drive the engine from external MIDI clock
- integrate with an audio scheduling layer

## 8. Worker-aware transport behavior

When the runtime executes in a Web Worker:

- all transport logic must remain self-contained inside the worker
- the main thread should send commands and receive state/events
- fine-grained tick scheduling should not require synchronous main-thread calls
- event emission should be batchable to reduce postMessage overhead

## 9. Non-goals for transport v1

The following are intentionally outside the core transport contract:

- browser audio graph scheduling specifics
- AudioWorklet implementation details
- sample-accurate DSP rendering guarantees
- network clock consensus
