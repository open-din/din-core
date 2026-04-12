# DocumentHandle and query API

## Behavior

- [`DocumentHandle`](../../crates/din-document/src/handle.rs) is built only when [`ValidationReport::accepted`](../../crates/din-document/src/report.rs) is true (`try_new` / `from_validated_arc`).
- Read-only accessors mirror `v2/specs/03-core-api.md` §4: scenes, collections, per-scene routes, timeline, and `graph(scene_id)` via [`SceneGraphView`](../../crates/din-document/src/handle.rs).

## Tests

- Unit tests in `crates/din-document/src/handle.rs` (default scene id, rejected report).
