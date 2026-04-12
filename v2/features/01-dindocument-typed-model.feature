@task @tdd @docs_v2 @critical-path
Feature: DinDocument typed model (serde) aligned with DinDocument v1 core
  # Status: completed (archived under v2/features)
  # Spec: v2/specs/03-core-api.md §1–2, v2/specs/02-crate-layout.md §2
  # Normative: open-din/v2/schema/din-document.core.schema.json
  # Target crate: crates/din-document
  # docs_v2: docs_v2/01-dindocument-typed-model.md
  # Tests: crates/din-document/tests/parse_corpus.rs — minimal + orchestrated-scene fixtures

  Scenario: Round-trip serde for minimal document
    Given fixture fixtures/din-document-v1/minimal.din.json
    When the JSON is deserialized into DinDocument
    Then format is open-din/din-document
    And version is 1
    And defaultSceneId is blank

  Scenario: Deserialize orchestrated scene with routes and timeline
    Given fixture fixtures/din-document-v1/orchestrated-scene.din.json
    When the JSON is deserialized into DinDocument
    Then scene main contains dsp samplerA and verbA
    And routes include a number route with transform
