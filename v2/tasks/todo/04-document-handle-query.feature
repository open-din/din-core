@task @tdd @docs_v2 @critical-path
Feature: DocumentHandle indexing and read-only query API
  # Spec: v2/specs/03-core-api.md §4, v2/specs/04-validation-and-query.md §6
  # Target crate: crates/din-document or crates/din-core (integration later)
  # docs_v2: docs_v2/04-document-handle-query.md
  # Tests first: query default_scene, scenes(), scene(id) errors

  Scenario: Build handle from accepted document
    Given a validated DinDocument
    When a DocumentHandle is built
    Then queries are deterministic and read-only

  Scenario: Resolve default scene
    When default_scene is queried
    Then the scene id matches defaultSceneId
