@task @tdd @docs_v2 @critical-path
Feature: TransportController, SequencerController, BridgeController
  # Spec: v2/specs/05-runtime-transport.md §2–4, §5–7
  # Target crate: crates/din-core
  # docs_v2: docs_v2/07-transport-sequencer-bridge.md
  # Tests first: transport state transitions; no alloc-per-tick in hot path (benchmark or guard test)

  Scenario: Transport supports start stop pause resume seek
    Given an active RuntimeSession
    When transport commands are applied
    Then transport state reflects the spec operations

  Scenario: Sequencer trigger and retrigger
    When sequencer commands are applied for scene timeline definitions
    Then sequencer state updates deterministically
