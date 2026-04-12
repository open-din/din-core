@task @tdd @docs_v2 @critical-path
Feature: Worker message dispatcher and batching rules
  # Spec: v2/specs/07-worker-model.md
  # Target crate: crates/din-wasm
  # docs_v2: docs_v2/09-worker-message-contract.md
  # Tests first: message round-trip serialization; batched events/drain

  Scenario: Dispatch document and runtime message families
    Given worker messages document/open runtime/create transport/tick
       When handle_message processes each
    Then responses carry structured success or failure