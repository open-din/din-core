# Core Rust API

## 1. Input model

The core crate accepts document content already obtained by the platform.

Supported conceptual inputs:

- UTF-8 JSON text
- parsed JSON value
- typed `DinDocument` object

The core crate must not require a file path.

## 2. Primary objects

Recommended top-level Rust-facing concepts:

- `DinDocument` — typed immutable document model
- `ValidationReport` — structured validation output
- `DocumentHandle` — indexed queryable document view
- `RuntimeSession` — mutable runtime state bound to one selected scene
- `TransportController` — transport API for the runtime
- `SequencerController` — sequencing API for the runtime
- `BridgeController` — signal bridge API for the runtime

## 3. Document lifecycle

### 3.1 Parse

The platform submits JSON content to the core.

Expected operation:

- parse text or value into `DinDocument`
- preserve structured diagnostics if parsing fails

### 3.2 Validate

The parsed document is validated against:

- root shape constraints
- closed vocabularies
- uniqueness rules
- reference resolution rules
- route graph rules
- enabled profiles

### 3.3 Index

If accepted, the document is indexed for fast access by id and by scope.

### 3.4 Query

The resulting document handle exposes read-only query APIs.

### 3.5 Runtime creation

A runtime session is created from:

- a validated document handle
- a selected scene id, or default scene id
- optional runtime configuration

## 4. Query requirements

A document handle must support:

- `default_scene()`
- `scene(scene_id)`
- `scenes()`
- `buffers()`
- `buffer_views()`
- `audio_sources()`
- `midi_sources()`
- `sample_slots()`
- `impulses()`
- `dsp_modules()`
- `scene_inputs(scene_id)`
- `scene_outputs(scene_id)`
- `scene_routes(scene_id)`
- `scene_dsp(scene_id)`
- `scene_transport(scene_id)`
- `scene_timeline(scene_id)`
- `scene_tracks(scene_id)`
- `scene_sequencers(scene_id)`
- `graph(scene_id)`

## 5. Resolution requirements

The core API must support internal resolution helpers such as:

- resolve scene DSP instance → DSP module
- resolve route endpoint → concrete typed endpoint
- resolve clip → MIDI source
- resolve track/sequencer ids inside a scene

## 6. Immutability boundary

The parsed and indexed document must be immutable.
Mutable behavior belongs in runtime session state, not in the document object.
