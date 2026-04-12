# Graph Report - /Users/veacks/Sites/open-din/din-core  (2026-04-12)

## Corpus Check
- Corpus is ~11,539 words - fits in a single context window. You may not need a graph.

## Summary
- 296 nodes · 322 edges · 28 communities detected
- Extraction: 94% EXTRACTED · 6% INFERRED · 1% AMBIGUOUS · INFERRED: 18 edges (avg confidence: 0.82)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_DinDocument typed model|DinDocument typed model]]
- [[_COMMUNITY_DocumentHandle query API|DocumentHandle query API]]
- [[_COMMUNITY_Runtime session & controllers|Runtime session & controllers]]
- [[_COMMUNITY_Validation & WASM surface|Validation & WASM surface]]
- [[_COMMUNITY_Parse-to-validate policy|Parse-to-validate policy]]
- [[_COMMUNITY_WASM bindgen shims|WASM bindgen shims]]
- [[_COMMUNITY_Worker & runtime docs|Worker & runtime docs]]
- [[_COMMUNITY_Crate roots & JS pkg|Crate roots & JS pkg]]
- [[_COMMUNITY_Fixture parse corpus|Fixture parse corpus]]
- [[_COMMUNITY_Scene route graph math|Scene route graph math]]
- [[_COMMUNITY_validate_document tests|validate_document tests]]
- [[_COMMUNITY_ValidationReport & IssueCode|ValidationReport & IssueCode]]
- [[_COMMUNITY_JSON parse errors|JSON parse errors]]
- [[_COMMUNITY_Profile gating (executionhost)|Profile gating (execution/host)]]
- [[_COMMUNITY_WASM parity tests|WASM parity tests]]
- [[_COMMUNITY_Skills & cross-repo contracts|Skills & cross-repo contracts]]
- [[_COMMUNITY_Graph view & matrix|Graph view & matrix]]
- [[_COMMUNITY_SceneGraphView bridge|SceneGraphView bridge]]
- [[_COMMUNITY_Route & endpoints|Route & endpoints]]
- [[_COMMUNITY_Normative v2 schema index|Normative v2 schema index]]
- [[_COMMUNITY_Host binding diagnostics|Host binding diagnostics]]
- [[_COMMUNITY_DEEP WASM policy|DEEP WASM policy]]
- [[_COMMUNITY_dinCoreVersion export|dinCoreVersion export]]
- [[_COMMUNITY_din-core lib root|din-core lib root]]
- [[_COMMUNITY_din-wasm lib root|din-wasm lib root]]
- [[_COMMUNITY_din-document lib root|din-document lib root]]
- [[_COMMUNITY_TransportController|TransportController]]
- [[_COMMUNITY_Document version constant|Document version constant]]

## God Nodes (most connected - your core abstractions)
1. `DocumentHandle` - 35 edges
2. `RuntimeSession` - 11 edges
3. `TransportController` - 9 edges
4. `validate_document` - 8 edges
5. `validate_document` - 8 edges
6. `SequencerController` - 7 edges
7. `parse_fixture()` - 7 edges
8. `validate_document()` - 7 edges
9. `din_document_wasm (WASM bindgen + worker dispatch)` - 6 edges
10. `crates/din-wasm` - 5 edges

## Surprising Connections (you probably didn't know these)
- `parse_document_json_str` --precedes_in_typical_pipeline--> `validate_document`  [INFERRED]
  docs/02-parse-pipeline.md → crates/din-document/src/validate.rs
- `DocumentHandle` --constructed_only_when_accepted--> `ValidationReport`  [EXTRACTED]
  docs/04-document-handle-query.md → crates/din-document/src/report.rs
- `docs/11-execution-profile.md` --documents_gate_for--> `IssueCode::UnsupportedProfileFeature`  [EXTRACTED]
  docs/11-execution-profile.md → crates/din-document/src/validate.rs
- `docs/12-host-binding-profile.md` --documents_resolution_rule_for--> `IssueCode::HostBindingUnresolved (unknown scene input/output)`  [EXTRACTED]
  docs/12-host-binding-profile.md → crates/din-document/src/validate.rs
- `docs/01-dindocument-typed-model.md` --documents--> `DinDocument`  [EXTRACTED]
  docs/01-dindocument-typed-model.md → crates/din-document/src/model.rs

## Hyperedges (group relationships)
- **Scene graph view builds route digraph then runs cycle/topology analysis** — handle_rs_SceneGraphView, graph_rs_SceneRouteGraph, graph_rs_route_analysis [EXTRACTED 0.95]
- **Tests assert WASM validation output matches native validate_document** — din_wasm_tests_wasm_rs, din_document_wasm_rs_module, parse_rs_parse_document [EXTRACTED 0.92]
- **Worker transport/tick mutates process-global RuntimeSession transport state** — din_document_wasm_rs_module, runtime_rs_RuntimeSession, runtime_rs_TransportController [EXTRACTED 0.90]
- **Optional profiles gate profile-specific payloads (execution DSP metadata; host-binding scene bindings)** — enum_document_profile, struct_dsp_module_execution, struct_scene_host_bindings, issue_unsupported_profile_feature [EXTRACTED 0.93]
- **Scene route graph checks: acyclic directed graph and no multi-writer fan-in to sinks** — fn_validate_scene_routes, fn_build_scene_route_graph, issue_route_cycle, issue_multiple_writers_sink [EXTRACTED 0.94]
- **WASM worker envelope: open document → create runtime session (default scene) → transport tick loop** — wasm_worker_dispatch, worker_family_document_open, worker_family_runtime_create, worker_family_transport_tick, type_runtime_session [EXTRACTED 0.89]
- **Validation surface: native validate_document vs WASM din_document_validate_json_impl parity on shared bytes** — n_validate_document, n_din_document_validate_json_impl, n_test_wasm_parity, n_fixture_minimal [INFERRED 0.96]
- **Route graph: build_scene_route_graph, SceneGraphView, cycle-related IssueCodes, invalid-route-cycle fixture** — n_build_scene_route_graph, n_scene_graph_view, n_issue_route_cycle, n_fixture_route_cycle [INFERRED 0.94]
- **Archived skills: patch-contract-change and registry-parity redirect to react-din / external registry; active work uses v2-refactor-task** — n_skill_patch_contract, n_skill_registry_parity, n_skill_v2_refactor, n_react_din_patch_authority [INFERRED 0.91]

## Communities

### Community 0 - "DinDocument typed model"
Cohesion: 0.04
Nodes (39): Asset, AudioFormat, AudioLayout, AudioSource, AutomationInterpolation, AutomationPoint, Buffer, BufferView (+31 more)

### Community 1 - "DocumentHandle query API"
Cohesion: 0.07
Nodes (5): default_scene_matches_default_scene_id(), DocumentHandle, DocumentHandleBuildError, index_by_id(), SceneGraphView

### Community 2 - "Runtime session & controllers"
Cohesion: 0.07
Nodes (8): BridgeController, RuntimeSession, RuntimeSessionError, SequencerController, session_rejects_unknown_scene(), transport_commands_and_tick_no_panic(), TransportCommand, TransportController

### Community 3 - "Validation & WASM surface"
Cohesion: 0.08
Nodes (26): crates/din-core, crates/din-document, crates/din-wasm, dinDocumentValidateJson, din_document_validate_json_impl, docs_v2/09-worker-message-contract.md, DocumentHandle, fixtures/din-document-v1/minimal.din.json (+18 more)

### Community 4 - "Parse-to-validate policy"
Cohesion: 0.11
Nodes (19): AGENTS.md (din-core), DOCUMENT_FORMAT (open-din/din-document), docs/01-dindocument-typed-model.md, docs/02-parse-pipeline.md, build_scene_route_graph, parse_document_json_str, validate_document, validate_scene_routes (+11 more)

### Community 5 - "WASM bindgen shims"
Cohesion: 0.17
Nodes (9): din_document_validate_json(), din_document_validate_json_impl(), runtime_envelope(), validation_report_to_value(), WasmDinDocumentHandle, WasmValidationIssueView, worker_dispatch_message_json(), worker_dispatch_message_json_impl() (+1 more)

### Community 6 - "Worker & runtime docs"
Cohesion: 0.12
Nodes (16): docs/04-document-handle-query.md, docs/06-runtime-session.md, docs/07-transport-sequencer-bridge.md, docs/09-worker-message-contract.md, UnknownScene error envelope, BridgeController (placeholder), DocumentHandle, RuntimeSession (+8 more)

### Community 7 - "Crate roots & JS pkg"
Cohesion: 0.19
Nodes (13): din-core crate root (re-exports document stack + runtime), din-document crate facade (exports), parse_corpus integration tests, WasmDinDocumentHandle, din_document_wasm (WASM bindgen + worker dispatch), WASM integration tests (parity + worker envelopes), DocumentHandle, parse_document_json_str / parse_document_json_value (+5 more)

### Community 8 - "Fixture parse corpus"
Cohesion: 0.38
Nodes (9): execution_descriptor_without_profile_rejected(), fixture(), host_binding_valid_accepted(), host_bindings_without_profile_rejected(), invalid_default_scene_rejected(), invalid_enum_fails_at_parse(), minimal_round_trip_parse(), orchestrated_scene_routes_and_transform() (+1 more)

### Community 9 - "Scene route graph math"
Cohesion: 0.33
Nodes (8): build_scene_route_graph(), dfs_cycle(), directed_graph_has_cycle(), orchestrated_scene_route_dag_has_topological_order(), route_endpoint_key(), SceneRouteGraph, topological_order(), two_node_cycle_detected()

### Community 10 - "validate_document tests"
Cohesion: 0.46
Nodes (7): cyclic_routes_rejected(), minimal_accepted(), orchestrated_accepted(), validate_document(), validate_host_binding_targets(), validate_scene_routes(), wrong_version_rejected()

### Community 11 - "ValidationReport & IssueCode"
Cohesion: 0.29
Nodes (3): IssueCode, ValidationIssue, ValidationReport

### Community 12 - "JSON parse errors"
Cohesion: 0.33
Nodes (3): malformed_json_fails(), parse_document_json_str(), ParseError

### Community 13 - "Profile gating (execution/host)"
Cohesion: 0.33
Nodes (7): docs/11-execution-profile.md, DocumentProfile, IssueCode::UnsupportedProfileFeature, DocumentProfile::Execution, DocumentProfile::HostBinding (host-binding), DspModule.execution (optional JSON), Scene.hostBindings (HostBindings)

### Community 14 - "WASM parity tests"
Cohesion: 0.33
Nodes (0): 

### Community 15 - "Skills & cross-repo contracts"
Cohesion: 0.33
Nodes (6): fixtures/din-document-v1/README, open-din/v2 (normative interchange), react-din patch JSON schema authority, patch-contract-change (ARCHIVED), registry-parity (ARCHIVED), v2-refactor-task

### Community 16 - "Graph view & matrix"
Cohesion: 0.5
Nodes (4): build_scene_route_graph, F05-S01..S02 (graph scenarios), RouteEndpoint, SceneGraphView

### Community 17 - "SceneGraphView bridge"
Cohesion: 0.67
Nodes (3): SceneRouteGraph, Route graph analysis (cycle detection + topological order), SceneGraphView

### Community 18 - "Route & endpoints"
Cohesion: 0.67
Nodes (3): RouteEndpoint (sceneInput, sceneOutput, dspPort, transport, track, sequencer, ...), Route (orchestration edge), TransportRouteMember (bpm, playing)

### Community 19 - "Normative v2 schema index"
Cohesion: 0.67
Nodes (3): docs/README.md (v2 doc index), fixtures/din-document-v1/, open-din/v2/schema/din-document.core.schema.json

### Community 20 - "Host binding diagnostics"
Cohesion: 1.0
Nodes (2): docs/12-host-binding-profile.md, IssueCode::HostBindingUnresolved (unknown scene input/output)

### Community 21 - "DEEP WASM policy"
Cohesion: 1.0
Nodes (1): DEEP: thin FFI/WASM, reuse Rust-native logic

### Community 22 - "dinCoreVersion export"
Cohesion: 1.0
Nodes (2): DIN_CORE_VERSION, dinCoreVersion

### Community 23 - "din-core lib root"
Cohesion: 1.0
Nodes (0): 

### Community 24 - "din-wasm lib root"
Cohesion: 1.0
Nodes (0): 

### Community 25 - "din-document lib root"
Cohesion: 1.0
Nodes (0): 

### Community 26 - "TransportController"
Cohesion: 1.0
Nodes (1): TransportController

### Community 27 - "Document version constant"
Cohesion: 1.0
Nodes (1): DOCUMENT_VERSION (=1)

## Ambiguous Edges - Review These
- `din_document_wasm (WASM bindgen + worker dispatch)` → `din_wasm.d.ts (TypeScript surface)`  [AMBIGUOUS]
  crates/din-wasm/pkg/din_wasm.d.ts · relation: conceptually_related_to
- `AGENTS.md (din-core)` → `README.md (legacy patch narrative)`  [AMBIGUOUS]
  README.md · relation: may_conflict_on_v1_patch_vs_dindocument_v2_emphasis

## Knowledge Gaps
- **104 isolated node(s):** `RuntimeSessionError`, `TransportCommand`, `BridgeController`, `WasmValidationIssueView`, `WorkerMessage` (+99 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Host binding diagnostics`** (2 nodes): `docs/12-host-binding-profile.md`, `IssueCode::HostBindingUnresolved (unknown scene input/output)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `DEEP WASM policy`** (2 nodes): `AGENTS.deep.md`, `DEEP: thin FFI/WASM, reuse Rust-native logic`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `dinCoreVersion export`** (2 nodes): `DIN_CORE_VERSION`, `dinCoreVersion`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `din-core lib root`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `din-wasm lib root`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `din-document lib root`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `TransportController`** (1 nodes): `TransportController`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Document version constant`** (1 nodes): `DOCUMENT_VERSION (=1)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `din_document_wasm (WASM bindgen + worker dispatch)` and `din_wasm.d.ts (TypeScript surface)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `AGENTS.md (din-core)` and `README.md (legacy patch narrative)`?**
  _Edge tagged AMBIGUOUS (relation: may_conflict_on_v1_patch_vs_dindocument_v2_emphasis) - confidence is low._
- **Why does `validate_document` connect `Parse-to-validate policy` to `Worker & runtime docs`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **What connects `RuntimeSessionError`, `TransportCommand`, `BridgeController` to the rest of the system?**
  _104 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `DinDocument typed model` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._
- **Should `DocumentHandle query API` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._
- **Should `Runtime session & controllers` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._