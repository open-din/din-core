Feature: Runtime session, transport, and sequencer
  As a runtime host
  I want a mutable session bound to one scene with transport and sequencer control
  So that playback state is driven by the host clock without platform I/O in core

  Background:
    Given runtime behavior is specified in v2/specs/05-runtime-transport.md
    And the core does not perform file or network I/O

  Scenario: Create a runtime session from a validated handle
    Given a validated document handle and a selected scene id
    When a runtime session is created
    Then the session exposes transport and sequencer controllers as specified

  Scenario: Transport stepping without hot-path JSON
    Given an active runtime session
    When the host advances transport with advance_to or clock events
    Then no full document re-materialization is required on each tick
