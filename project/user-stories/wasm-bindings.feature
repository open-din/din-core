Feature: WebAssembly binding crate
  As a browser host
  I want JavaScript-friendly bindings separate from din-core
  So that document and runtime APIs work on main thread or in a worker

  Background:
    Given the binding contract is v2/specs/06-wasm-bindings.md
    And din-core must not depend on wasm-bindgen

  Scenario: Load and validate from JSON text in WASM
    When WASM exports open a document from JSON text
    Then validation reports map to stable JS-facing diagnostics

  Scenario: Explicit disposal of long-lived handles
    Given document and runtime handles created in WASM
    When the host disposes a handle
    Then resources are released deterministically
