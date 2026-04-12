# din-core v2 documentation

English-only technical notes for the DinDocument v1 refactor tracked under `v2/specs` and `tasks/`.

Each page corresponds to a Gherkin task in `tasks/todo` (active) or `v2/features` (completed).

| Doc | Task | Crate |
|-----|------|--------|
| [01-dindocument-typed-model.md](01-dindocument-typed-model.md) | Typed model | `crates/din-document` |
| [02-parse-pipeline.md](02-parse-pipeline.md) | Parse pipeline | `crates/din-document` |
| [03-validation-and-diagnostics.md](03-validation-and-diagnostics.md) | Validation | `crates/din-document` |

Normative JSON: `open-din/v2/schema/din-document.core.schema.json` (workspace sibling). Example corpus: `fixtures/din-document-v1/` (synced from `open-din/v2/examples`).
