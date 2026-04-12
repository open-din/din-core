@task @tdd @docs_v2 @critical-path
Feature: WASM binding crate aligned with din-core-wasm spec
  # Spec: v2/specs/06-wasm-bindings.md
  # Target crate: crates/din-wasm (rename to din-core-wasm in a follow-up task if required)
  # docs_v2: docs_v2/08-wasm-bindings.md
  # Tests: wasm-bindgen tests where applicable; error code mapping

  Scenario: Export document open and validate to JavaScript
    When JSON text is passed across the WASM boundary
    Then validation diagnostics are plain JS objects with stable codes

  Scenario: Host can dispose long-lived handles
    When dispose is called on a WASM handle
    Then underlying Rust resources are dropped
