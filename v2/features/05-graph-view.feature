@task @tdd @docs_v2 @critical-path
Feature: Normalized graph view for scene routes
  # Spec: v2/specs/04-validation-and-query.md §7
  # Target crate: crates/din-document / crates/din-core
  # docs_v2: docs_v2/05-graph-view.md
  # Tests first: DAG order for orchestrated-scene; cycle detection fixture invalid-route-cycle

  Scenario: Expose nodes and edges for a scene graph
    Given an accepted scene with routes
    When graph(scene_id) is requested
    Then nodes and edges reflect route endpoints

  Scenario: Report cycle diagnostics when routes are cyclic
    Given an invalid cyclic route document
    When validation or graph build runs
    Then route_cycle issues are emitted
