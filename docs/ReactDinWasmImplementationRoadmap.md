# react-din <- din-core wasm Implementation Roadmap

## Scope

This roadmap turns the audit into an implementation sequence with PR-sized chunks, prioritizing P0 blockers first, then parity and hardening.

## Guiding constraints

- Keep `react-din` public exports stable.
- Keep `din-wasm` and `din-ffi` thin wrappers around `din-core` behavior.
- Land behavior behind feature flags until parity confidence is high.
- Add parity tests with each chunk; avoid big-bang runtime rewrites.

## PR chunk plan

### PR-1 (P0): Engine graph execution skeleton

**Goal**
- Move from node-sum rendering to graph-driven execution scaffolding.

**Files**
- `crates/din-core/src/engine.rs`
- `crates/din-core/src/graph.rs`
- `crates/din-core/tests/core.rs`

**Changes**
- Introduce runtime execution plan structures (`ExecutionPlan`, `ExecutionNode`, handle maps).
- Build deterministic traversal order from `CompiledGraph`.
- Route final output through nodes connected to `output`.
- Keep existing placeholder DSP node kernels initially, but execute them through graph wiring.

**Tests**
- Add topology-sensitive tests:
  - graph rewiring changes output
  - disconnected node no longer contributes to output
  - basic control connection influences target parameter

**Exit criteria**
- `render_block` behavior depends on graph connectivity (not global node summation).

---

### PR-2 (P0): Native `patch` node support (nested runtime)

**Goal**
- Remove hard rejection of `NodeKind::Patch` and execute nested patch graphs.

**Files**
- `crates/din-core/src/engine.rs`
- `crates/din-core/src/graph.rs` (if slot normalization helpers are needed)
- `crates/din-core/src/lib.rs` (error surface cleanup)
- `crates/din-core/tests/core.rs`
- `crates/din-wasm/tests/wasm.rs`
- `crates/din-ffi/tests/ffi.rs`

**Changes**
- Replace `UnsupportedNativeNode` fast-fail path with nested engine/subgraph execution.
- Implement slot mapping for parent/child `in`, `in:*`, `out`, `out:*`.
- Add recursion protection (depth cap + visited patch identity tracking).

**Tests**
- Existing rejection tests become success-path tests.
- Add recursion guard tests.
- Add nested patch signal propagation tests.

**Exit criteria**
- Canonical fixture with `patch-1` initializes and renders in core, wasm, and ffi.

---

### PR-3 (P1): Trigger/event semantics in render loop

**Goal**
- Make `trigger_event` tokens affect runtime behavior.

**Files**
- `crates/din-core/src/engine.rs`
- `crates/din-core/tests/core.rs`
- `crates/din-wasm/tests/wasm.rs`

**Changes**
- Consume `event_tokens` in trigger/gate node paths.
- Add token edge detection utility in engine state.
- Wire trigger propagation into control/trigger connections.

**Tests**
- Trigger token change produces observable output/state transition.
- Idempotent behavior for unchanged token.

**Exit criteria**
- Event-triggered behavior is no longer a no-op in runtime.

---

### PR-4 (P1): MIDI scheduling and semantic expansion

**Goal**
- Upgrade MIDI from note on/off state to sample-aware event application.

**Files**
- `crates/din-core/src/engine.rs`
- `crates/din-core/src/transport.rs` (only if timing helpers are needed)
- `crates/din-core/tests/core.rs`
- `crates/din-wasm/src/lib.rs` (surface remains thin)
- `crates/din-wasm/tests/wasm.rs`

**Changes**
- Apply queued MIDI events by `frame_offset` within each render block.
- Add CC handling and transport-clock relevant message handling.
- Keep host WebMIDI management in `react-din`; only consume normalized engine events.

**Tests**
- Intra-block offset changes output timing.
- CC updates affect mapped controls.
- Clock/start/stop signals update transport-linked behavior where applicable.

**Exit criteria**
- `frame_offset` is behaviorally effective and covered by tests.

---

### PR-5 (P1): Orchestration node coverage

**Goal**
- Add runtime semantics for high-impact non-audio orchestration nodes.

**Files**
- `crates/din-core/src/engine.rs`
- `crates/din-core/src/graph.rs` (classification/tags if needed)
- `crates/din-core/tests/core.rs`

**Changes**
- Implement/expand runtime behavior for:
  - `transport`
  - `stepSequencer`
  - `pianoRoll`
  - `eventTrigger`
  - `voice`
  - `midiNote` / `midiCC` inputs
- Keep data-node helper logic shared with existing core functions.

**Tests**
- Sequencer drives gate/trigger downstream.
- Transport-linked nodes advance deterministically.
- Voice/gate pathways affect oscillator/envelope paths.

**Exit criteria**
- Core orchestration flows used by `PatchRenderer` are represented in engine semantics.

---

### PR-6 (P2): wasm adapter readiness for react-din integration

**Goal**
- Make wasm runtime consumption practical from React host code.

**Files**
- `crates/din-wasm/src/lib.rs`
- `crates/din-wasm/pkg/din_wasm.d.ts` (generated artifact update)
- `crates/din-wasm/tests/wasm.rs`

**Changes**
- Keep wrappers thin; expose missing runtime entry points only where necessary:
  - asset loading API (if retained in core runtime)
  - optional buffer-reuse render API (if introduced in core)
  - richer typed snapshots for runtime config/state if needed

**Tests**
- New exported methods covered in wasm tests.
- No logic duplication beyond adapter mapping.

**Exit criteria**
- wasm API is sufficient for a React-side adapter layer without workaround hacks.

---

### PR-7 (P2): Cross-repo adapter + fallback gate (react-din)

**Goal**
- Integrate wasm backend behind a feature flag while preserving API contracts.

**Files (react-din)**
- `src/core/AudioProvider.tsx`
- `src/patch/PatchRenderer.tsx`
- `src/transport/TransportContext.tsx`
- `src/midi/MidiProvider.tsx`
- `src/midi/runtime.ts` (bridge points only)
- new internal adapter module(s), e.g. `src/runtime/wasm/*`
- parity tests in `tests/library/*`

**Changes**
- Introduce internal runtime adapter interface and wasm implementation.
- Route inputs/events/MIDI to wasm runtime when feature flag is enabled.
- Keep legacy runtime as fallback per unsupported capability/node kind.

**Tests**
- Existing public API tests remain green under default mode.
- Add dual-mode parity tests (legacy vs wasm backend).

**Exit criteria**
- `PatchRenderer` can run with wasm backend on selected scenarios without API changes.

---

### PR-8 (P2): Performance hardening and CI parity gates

**Goal**
- Make wasm runtime production-ready and continuously validated.

**Files**
- `crates/din-core/src/engine.rs` (allocation and hot-path tuning)
- benchmark harness files (new, as needed)
- CI workflow updates in relevant repos
- parity test suites in `react-din` and `din-core`

**Changes**
- Reduce block-loop allocations and redundant copies.
- Add perf benchmarks and threshold checks.
- Add parity CI matrix (functional + timing + perf).

**Tests and metrics**
- Behavioral parity suite pass rate.
- Timing error bounds in samples.
- CPU/block and allocations/block budget checks.

**Exit criteria**
- CI enforces parity and performance budgets for ongoing changes.

## Validation checklist per PR

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

When changing react-din integration chunks:
- `npm run lint`
- `npm run typecheck`
- `npm run ci:check`

## Risk control

- Keep each PR single-purpose and reversible.
- Prefer additive flags and compatibility shims until parity confidence is proven.
- Do not remove legacy runtime paths until parity KPIs are stable.
