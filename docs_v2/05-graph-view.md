# Route graph view

## Behavior

- [`build_scene_route_graph`](../../crates/din-document/src/graph.rs) deduplicates [`RouteEndpoint`](../../crates/din-document/src/model.rs) nodes and records directed edges.
- [`SceneGraphView`](../../crates/din-document/src/handle.rs) exposes topological order when acyclic and flags cycles.
- Validation emits [`IssueCode::RouteCycle`](../../crates/din-document/src/report.rs) and [`IssueCode::MultipleWritersToSink`](../../crates/din-document/src/report.rs) when applicable.

## Fixtures

- `fixtures/din-document-v1/invalid-route-cycle.din.json` — two endpoint cycle (scene I/O loop).
