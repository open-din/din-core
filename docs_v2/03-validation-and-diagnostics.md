# Validation and diagnostics

**Task:** `v2/features/03-validation-and-diagnostics.feature`  
**Crate:** [`crates/din-document`](../crates/din-document)

## Purpose

After successful parse, [`validate_document`](../crates/din-document/src/validate.rs) applies **semantic** rules from `v2/specs/04-validation-and-query.md` (initial subset).

## Report

- `ValidationReport`: `accepted` flag and `issues: Vec<ValidationIssue>`.
- Each `ValidationIssue` has `code: IssueCode`, `message`, optional `path` (JSON-pointer style).

## Stable codes (initial)

| `IssueCode` | `as_str()` | Meaning |
|-------------|------------|---------|
| `InvalidFormatVersion` | `invalid_format_version` | `format` ≠ `open-din/din-document` or `version` ≠ `1` |
| `UnresolvedReference` | `unresolved_reference` | `defaultSceneId` does not match any `scenes[].id` |
| `ParseError` | `parse_error` | Reserved for callers mapping parse failures into reports |

## Tests

- `validate::tests::minimal_accepted`
- `parse_corpus::invalid_default_scene_rejected`
- Invalid enum fixtures fail at **parse** time (serde), not validation.

## Next steps

Expand validation for routes (fan-in, cycles), profile gates, and full issue families per `v2/specs/04-validation-and-query.md` §5.
