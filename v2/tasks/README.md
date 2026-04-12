# Task board (Gherkin)

English only. See `project/skills/v2-refactor-task/SKILL.md` and `.cursor/rules/din-core-v2-task-workflow.mdc`.

## Folders

| Folder | Meaning |
|--------|---------|
| `todo/` | Ready to implement |
| `doing/` | Work in progress |
| `done/` | Implemented; awaiting final `docs_v2` sync before archive |
| `../v2/features/` | Completed tasks (archive) |

## Critical path (sequential)

`01-model` → `02-parse` → `03-validation` (completed: see `v2/features/01-*.feature` … `03-*.feature`) → `04-document-handle` → `05-graph-view` → `06-runtime-session` → `07-transport-sequencer-bridge` → `08-wasm-bindings` → `09-worker-messages` → `10-cross-target-parity`

## Parallel-safe (after shared types land)

- Corpus expansion: each `fixtures/din-document-v1/` invalid example as its own test (tag `@parallel-safe` on tasks).
- Diagnostics golden tests vs graph algorithms, when feature flags isolate crates cleanly.
- `docs_v2` pages once the API surface for a task is stable.
