Feature: Errors, events, and cross-target compatibility
  As a spec maintainer
  I want consistent diagnostics and compatibility rules across native and WASM
  So that the same document behaves equivalently on every target

  Background:
    Given rules in v2/specs/08-errors-events-and-versioning.md
    And DinDocument v1 container versioning in open-din/v2/README.md

  Scenario: Treat documents as untrusted input
    When arbitrary JSON is loaded
    Then the core does not assume executable artifacts or devices are safe or available

  Scenario: Validation parity across targets
    Given the same document bytes
    When validated on native Rust and on WASM
    Then acceptance and stable error codes match
