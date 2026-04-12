# SKILL: v2-refactor-task

## REPO

`din-core`

## WHEN TO USE

- Implementing or reviewing work tracked as Gherkin tasks under `tasks/todo`, `tasks/doing`, or `tasks/done`.
- Refactoring toward DinDocument v1 and `v2/specs`, using `open-din/v2` as the normative document format reference.

## STEPS

1. Read `AGENTS.md`, `project/ROUTE_CARD.json`, and this skill.
2. Pick **one** task file from `tasks/todo/*.feature` and move it to `tasks/doing/`.
3. **Documentation (smallest context first, only cited paths):**
   - `din-core-specs/` **only if** the task cites legacy dossier sections.
   - `docs_v2/<task-slug>.md` if it exists.
   - `v2/specs/*.md` **only** files referenced by the task.
   - `v2/user-stories/*.feature` **only** stories the task links or needs.
   - `open-din/v2` — schema, examples, or markdown **cited in the task** (and `fixtures/din-document-v1/` when the task uses examples).
4. **TDD**: add failing tests first (unit + integration against `fixtures/din-document-v1/` where applicable), then implement.
5. Add or update **rustdoc** (`///`) for all new public API in scope.
6. Add or update **`docs_v2/<task-slug>.md`** describing behavior, error codes, fixtures, and link to the Gherkin task path.
7. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
8. Move the task file from `tasks/doing/` to `tasks/done/`, then to **`v2/features/`** once documentation is complete.

## CONSTRAINTS

- **English only** for user stories, tasks, and `docs_v2`.
- Breaking changes to **react-din patch** contracts are out of scope unless coordinated; this repo’s live surface is **DinDocument v2** and minimal runtime session.

## VALIDATION

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
