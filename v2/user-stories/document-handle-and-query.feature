Feature: Document handle and read-only queries
  As a tooling author
  I want an indexed read-only view of a validated document
  So that I can inspect scenes, collections, and routes deterministically

  Background:
    Given the query API is defined in v2/specs/03-core-api.md sections 4 and 5
    And queries are side-effect free per v2/specs/04-validation-and-query.md section 6

  Scenario: Query default scene id
    Given an accepted document with defaultSceneId set
    When the document handle is asked for the default scene
    Then the id matches defaultSceneId

  Scenario: List scenes
    Given an accepted document with one or more scenes
    When the handle lists scenes
    Then every scene id in the document is returned exactly once

  Scenario: Immutability boundary
    Given an indexed document handle
    When queries run concurrently on the same handle
    Then no mutation of document data occurs through the query API
