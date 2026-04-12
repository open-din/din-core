# Validation and Query Requirements

## 1. Validation model

Validation must be split into at least two conceptual phases:

- structural validation
- semantic validation

## 2. Structural validation

Structural validation checks:

- required root fields
- allowed field types
- one-of / all-of constraints
- closed enum vocabularies
- field-level invariants

## 3. Semantic validation

Semantic validation checks:

- `defaultSceneId` resolves to an existing scene
- duplicate ids in a uniqueness scope are rejected
- route endpoint references resolve
- scene DSP instances reference existing modules
- timeline clips reference existing MIDI sources
- route type matching is exact
- `transform` appears only on number routes
- fan-in is rejected
- route cycles are rejected
- unsupported required extensions are rejected
- enabled profile semantics are enforced

## 4. Diagnostics

Validation output must include:

- accepted or rejected flag
- structured errors
- warnings
- stable error code per issue category
- logical path or scope reference
- human-readable message

## 5. Canonical issue categories

Recommended issue families:

- parse errors
- invalid format/version
- missing required field
- invalid enum value
- duplicate id
- unresolved reference
- invalid route endpoint usage
- route type mismatch
- forbidden transform
- multiple writers to a sink
- route cycle
- unsupported required extension
- unsupported profile feature
- profile capability mismatch

## 6. Query behavior

Query APIs must be:

- read-only
- deterministic
- side-effect free
- stable across native and WASM targets

## 7. Graph view

The graph API must expose at minimum:

- nodes
- edges
- node kinds
- endpoint references
- topological ordering when acyclic
- cycle diagnostics when invalid

The graph view should normalize the route system into an inspection-friendly representation.
