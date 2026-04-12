@task @tdd @docs_v2 @critical-path
Feature: Native vs WASM validation and query parity
  # Spec: v2/specs/08-errors-events-and-versioning.md §4
  # Target: integration tests in din-document + din-wasm
  # docs_v2: docs_v2/10-cross-target-parity.md
  # Tests first: same fixture bytes on both targets

  Scenario: Same document produces matching acceptance on native and WASM
    Given a corpus of valid fixtures
    When validate on each target
    Then accepted flag and issue codes match
