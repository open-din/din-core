@task @tdd @docs_v2 @parallel-safe
Feature: Host binding profile parsing and validation gates
  # Normative: open-din/v2/din-document-host-binding-profile.md, schema/din-document.host-binding.schema.json
  # Target crate: crates/din-document (feature host-binding-profile)
  # docs_v2: docs_v2/12-host-binding-profile.md
  # Depends on: 01–03 complete

  Scenario: Validate host binding surfaces when profile is declared
    Given profiles includes host-binding
    When scene host bindings reference declared surfaces
    Then validation accepts or rejects per host-binding rules
