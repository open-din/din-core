# Exhaustive react-din vs din-core/din-wasm Gap Audit

## Goal

Build a complete parity map between `react-din` runtime requirements and current `din-core`/`din-wasm` capabilities, then define the critical path to use `din-core wasm` as the engine backend without breaking the public API contract.

## 1) Definitive capability -> status matrix

Legend:
- `implemented`: usable coverage for the target behavior
- `partial`: exists but incomplete or semantically different
- `missing`: not implemented
- `incompatible`: explicitly conflicts with target behavior

### 1.1 Patch/schema/graph contract

| Capability | React requirement | din-core/wasm status | Evidence |
|---|---|---|---|
| Patch v1 validation/migration | Strict validation + transparent migration | implemented | `react-din/src/patch/PatchRenderer.tsx`, `react-din/src/patch/document/handles.ts`, `din-core/crates/din-wasm/src/lib.rs` (`validate_patch`, `migrate_patch`) |
| Connection classification (audio/transport/trigger/control) | Required for runtime semantics | implemented (compile only) | `din-core/crates/din-core/src/graph.rs`, `din-core/crates/din-core/tests/core.rs` (`compiled_graph_classifies_connections`) |
| Runtime use of classified graph | Graph-driven rendering (not node-isolated rendering) | missing | `din-core/crates/din-core/src/engine.rs` (`render_block` does not use `CompiledGraph` connection buckets) |
| Dynamic handles (switch in_N, matrix in/outN, patch slots) | Required by react-din patch graphs | partial | Contract-level support exists (`react-din/src/patch/document/handles.ts`, `din-core/crates/din-patch/src/document.rs`), runtime execution does not consume them semantically |

### 1.2 Audio/runtime execution

| Capability | React requirement | din-core/wasm status | Evidence |
|---|---|---|---|
| Engine creation from patch | Runtime instantiation from migrated patch | implemented | `din-core/crates/din-wasm/src/lib.rs` (`AudioRuntime::new/fromPatch`) |
| Audio block rendering | `PatchRenderer` expects graph semantics | partial | `din-core/crates/din-core/src/engine.rs` renders blocks but sums per-node synthetic output |
| `patch` node (nested patch) runtime support | Used in production (`patchInline`, `patchAsset`) | incompatible | `Engine::new` rejects `NodeKind::Patch` (`din-core/crates/din-core/src/engine.rs`); rejection tests in core/wasm/ffi |
| Connection-driven audio signal routing | Required for React parity | missing | `din-core/crates/din-core/src/engine.rs` does not evaluate source->target handles |
| Runtime asset consumption (sampler/convolver) | React resolves `assetRoot` and loads assets | partial | `Engine::load_asset` exists, but render path does not read loaded assets; no exposed wasm asset-loading API on `AudioRuntime` |

### 1.3 Transport / timeline / events

| Capability | React requirement | din-core/wasm status | Evidence |
|---|---|---|---|
| Transport object with bpm/signature/swing/tick | Required by transport hooks | implemented (object-level) | `din-core/crates/din-core/src/transport.rs`, `din-core/crates/din-wasm/src/lib.rs` (`TransportRuntime`) |
| React-equivalent `raf`/`manual` behavior | React has raf scheduler + manual `update` mode | partial | `react-din/src/transport/TransportContext.tsx` vs `din-core` `Raf/Tick` model, no React scheduler integration |
| Transport->sequencer/pianoRoll/midiPlayer execution | Required by patch graph semantics | missing in engine | transport connections are classified in graph, not consumed in `engine.rs` render |
| Effective `trigger_event` influence in render | EventTrigger drives gate/trigger and MIDI behavior | missing | `trigger_event` stores token (`event_tokens`) and is not used in rendering |

### 1.4 MIDI

| Capability | React requirement | din-core/wasm status | Evidence |
|---|---|---|---|
| Full WebMIDI runtime (ports/state/clock/send/receive) | `createMidiRuntime`, `useMidi`, `MidiTransportSync` | missing in core/wasm | Exists in `react-din/src/midi/runtime.ts` only |
| MIDI engine input ingestion | note events affect runtime | partial | `Engine::push_midi` updates note/gate state and stores message |
| Sample-accurate scheduling via `frame_offset` | Required for intra-block precision | missing | `frame_offset` is stored but not consumed during synthesis |
| CC and clock/start/stop runtime semantics | Required for midiCC and transport sync parity | missing | `push_midi` handles note on/off only |
| MIDI output nodes (note/cc/sync) | patch nodes `midiNoteOutput/midiCCOutput/midiSync` | missing | React runtime routes these in `PatchRenderer`; no equivalent in `din-core` runtime |

### 1.5 react-din node kind parity

#### Audio nodes
- **Core audio kinds** (`osc`, `gain`, `filter`, `delay`, `reverb`, etc.): `partial`
  - Present in `engine.rs` branches, but with placeholder/synthetic behavior, not graph-semantic DSP.
- **`output`/`mixer`/routing nodes**: `partial`
  - Treated as gain/noise-like output contributions, not topology-aware bus routing.

#### Non-audio orchestration nodes
- **`transport`, `stepSequencer`, `pianoRoll`, `eventTrigger`, `voice`, `adsr`**: `missing/partial`
  - Contract-level support exists; runtime orchestration semantics are absent or simplified.
- **`midiNote`, `midiCC`, `midi*Output`, `midiSync`, `midiPlayer`**: `missing`
  - No equivalent runtime execution path in `din-core` engine.
- **Data nodes (`math/compare/mix/clamp/switch`)**: `partial`
  - Helpers exist in core/wasm, but no graph-based control evaluation in engine render.
- **`input`, `uiTokens`, `patch`**: `partial/incompatible`
  - Inputs are supported (`set_input`), `uiTokens` has no runtime semantics, `patch` is explicitly rejected.

### 1.6 Public react-din API surface to preserve

Migration-critical exports:
- Core: `AudioProvider`, `useAudio`
- Transport: `TransportProvider`, `useTransport`, `useBeat`, `useStep`, `useBar`, `usePhrase`
- MIDI: `createMidiRuntime`, `MidiProvider`, `useMidi`, `useMidiNote`, `useMidiCC`, `useMidiClock`, `MidiTransportSync`
- Patch: `Patch`, `PatchRenderer`, `PatchOutput`, `importPatch`, `graphDocumentToPatch`, `patchToGraphDocument`, `migratePatchDocument`

Evidence: `react-din/src/index.ts`.

## 2) P0 runtime blockers: minimum required din-core changes

### P0-A Add `patch` node execution (nested patch runtime)

Current blocker:
- Engine creation hard-fails on `NodeKind::Patch` (`UnsupportedNativeNode`).

Minimum change set:
1. Replace hard rejection with nested compiled-subgraph execution.
2. Add nested execution context:
   - slot mapping for `in`/`in:*` and `out`/`out:*`
   - propagation of inputs/events/midi into child runtime
3. Add recursion protection policy (max depth + cycle detection), aligned with React behavior (`patch.spec.tsx` recursive-inline rejection semantics).

Acceptance criteria:
- Patch fixtures containing `patch-1` no longer fail in `Engine::new`.
- Existing core/wasm/ffi tests that currently expect rejection are replaced by successful nested rendering tests.

### P0-B Implement graph-driven rendering (audio/control/trigger/transport)

Current blocker:
- `render_block` sums node outputs without applying compiled connections.

Minimum change set:
1. Introduce a per-frame (or block-with-state) execution plan on top of `CompiledGraph`.
2. Evaluate source->target handle flows across all connection classes:
   - audio (`audio_connections`)
   - control (`control_connections`)
   - trigger/gate (`trigger_connections`)
   - transport (`transport_connections`)
3. Emit output from nodes connected to `output` path only (remove global sum of all nodes).

Acceptance criteria:
- Simple parity tests (osc->gain->output, data->param modulation) produce expected behavior when graph wiring changes.

## 3) React -> WASM adapter contract (API-stable migration)

### Principle

Keep the public `react-din` facade unchanged and progressively switch runtime internals to a wasm-backed adapter behind feature flags.

### 3.1 Core audio adapter

- Keep `AudioProvider` public API unchanged.
- In wasm mode:
  - create/manage `AudioRuntime` (`din-wasm`) for active patch
  - bridge via AudioWorklet (temporary ScriptProcessor fallback if needed)
  - map `setInput`/`triggerEvent` and MIDI ingestion to wasm runtime

Proposed internal adapter interface:
- `createWasmEngineAdapter({ sampleRate, channels, blockSize, patchJson })`
- `setInput(key, value)`
- `triggerEvent(key, token)`
- `pushMidi(status, data1, data2, frameOffset)`
- `render(outputBuffer?)`
- `dispose()`

### 3.2 Patch adapter

- Keep `PatchRenderer` props stable (`patch`, `includeProvider`, `assetRoot`, `midi`, interface props).
- Target pipeline:
  1. `migratePatchDocument` + validation (unchanged ownership)
  2. serialize patch -> `AudioRuntime.fromPatch`
  3. map interface inputs/events/midi bindings into wasm runtime calls
  4. fallback to legacy React runtime per unsupported capability/node kind

### 3.3 MIDI adapter

- Keep `createMidiRuntime` and `MidiProvider` as JS/WebMIDI runtime.
- Add runtime bridge:
  - inbound MIDI -> wasm `pushMidi`
  - outbound MIDI remains in existing React components until wasm parity is complete

### 3.4 Transport adapter

- Keep `TransportProvider/useTransport` API unchanged.
- Transition mode:
  - React scheduler remains source of truth for hooks/UI
  - transport ticks/position are mirrored to wasm (or sourced from wasm later via feature flags)

### Compatibility invariants

- No public export breakage in `react-din/src/index.ts`.
- TypeScript signatures remain stable for hooks/providers.
- Granular fallback by capability/node kind to avoid all-or-nothing regressions.

## 4) Cross-validation strategy (functional + timing + perf)

### 4.1 Parity test matrix

#### A) Golden behavioral parity (legacy JS runtime vs WASM runtime)

- Build a dual-engine harness:
  - same `PatchDocument`
  - same input/event/MIDI timeline
  - compare output envelopes/features within tolerances
- Target scenarios:
  - simple audio chains (osc/filter/gain/output)
  - modulation (LFO/data->param)
  - trigger/event (`eventTrigger` -> note output behavior)
  - transport + sequencer/pianoRoll
  - nested patch execution

#### B) MIDI/timing parity

- Scenarios:
  - note on/off with intra-block offsets (`frame_offset`)
  - dense CC streams
  - clock/start/stop/continue with BPM drift
- Metrics:
  - timing error in samples
  - event-to-audio latency
  - gate/frequency coherence at block boundaries

#### C) WASM production readiness/perf

- Offline benchmarks:
  - CPU per block across patch sizes
  - allocations per block
  - JS<->WASM copy cost (`Float32Array` churn)
- Initial targets:
  - no per-frame hot-path allocations
  - stable CPU under MIDI + transport load

### 4.2 Recommended delivery order

#### P0 (engine blockers)
1. Add `patch` node runtime support
2. Implement semantic graph execution (audio/control/trigger/transport)

#### P1 (behavioral parity)
3. Consume `trigger_event` during render
4. Implement sample-accurate MIDI + CC/clock/start-stop semantics
5. Cover orchestration node kinds (`transport`, `sequencer`, `eventTrigger`, `voice`, `midi*`)

#### P2 (React integration + hardening)
6. Complete React<->WASM adapter with feature flags/fallback
7. Reduce allocations/copies in runtime loop
8. Run CI parity campaign (functional + timing + perf)

### 4.3 KPI tracking

- `% of react-din node kinds with semantic execution in din-core`
- `% of react-din patch/midi/transport tests passing with wasm backend`
- `max timing error (samples)` on MIDI/event suites
- `allocations per render block` and `CPU/block` on reference patches

## Appendix: key evidence references

- React runtime contracts:
  - `react-din/src/index.ts`
  - `react-din/src/patch/PatchRenderer.tsx`
  - `react-din/src/patch/document/handles.ts`
  - `react-din/src/transport/TransportContext.tsx`
  - `react-din/src/midi/runtime.ts`
  - `react-din/tests/library/patch.spec.tsx`

- din-core/wasm implementation status:
  - `din-core/crates/din-core/src/engine.rs`
  - `din-core/crates/din-core/src/graph.rs`
  - `din-core/crates/din-core/tests/core.rs`
  - `din-core/crates/din-wasm/src/lib.rs`
  - `din-core/crates/din-wasm/pkg/din_wasm.d.ts`
  - `din-core/crates/din-wasm/tests/wasm.rs`
  - `din-core/crates/din-ffi/tests/ffi.rs`
