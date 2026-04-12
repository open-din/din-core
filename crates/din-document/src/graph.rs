//! Normalized route graph for a scene (inspection, topological order, validation helpers).

use crate::model::{RouteEndpoint, Scene};
use std::collections::HashMap;

/// Directed route graph for one scene: unique [`RouteEndpoint`] nodes and route edges.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneRouteGraph {
    /// Stable order of endpoints (index is the graph node id).
    pub endpoints: Vec<RouteEndpoint>,
    /// Directed edges `from` → `to` as endpoint indices.
    pub edges: Vec<(usize, usize)>,
}

/// Stable string key for deduplicating [`RouteEndpoint`] values.
pub fn route_endpoint_key(endpoint: &RouteEndpoint) -> String {
    ::serde_json::to_string(endpoint).expect("RouteEndpoint serializes to JSON")
}

/// Builds a deduplicated directed multigraph from [`Scene::routes`].
pub fn build_scene_route_graph(scene: &Scene) -> SceneRouteGraph {
    let mut key_to_index: HashMap<String, usize> = HashMap::new();
    let mut endpoints: Vec<RouteEndpoint> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();

    let mut node_index = |ep: &RouteEndpoint| -> usize {
        let key = route_endpoint_key(ep);
        if let Some(&ix) = key_to_index.get(&key) {
            return ix;
        }
        let ix = endpoints.len();
        key_to_index.insert(key, ix);
        endpoints.push(ep.clone());
        ix
    };

    for route in &scene.routes {
        let from = node_index(&route.from);
        let to = node_index(&route.to);
        edges.push((from, to));
    }

    SceneRouteGraph { endpoints, edges }
}

/// Returns `true` if the directed graph contains a cycle.
pub fn directed_graph_has_cycle(node_count: usize, edges: &[(usize, usize)]) -> bool {
    if node_count == 0 {
        return false;
    }
    let mut adj = vec![Vec::new(); node_count];
    for &(f, t) in edges {
        if f < node_count && t < node_count {
            adj[f].push(t);
        }
    }
    let mut color = vec![0u8; node_count];
    for start in 0..node_count {
        if color[start] == 0 && dfs_cycle(start, &adj, &mut color) {
            return true;
        }
    }
    false
}

fn dfs_cycle(u: usize, adj: &[Vec<usize>], color: &mut [u8]) -> bool {
    color[u] = 1;
    for &v in &adj[u] {
        match color[v] {
            0 => {
                if dfs_cycle(v, adj, color) {
                    return true;
                }
            }
            1 => return true,
            _ => {}
        }
    }
    color[u] = 2;
    false
}

/// Kahn topological ordering when the graph is acyclic; `None` if a cycle exists or nodes empty.
pub fn topological_order(node_count: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    if node_count == 0 {
        return Some(Vec::new());
    }
    if directed_graph_has_cycle(node_count, edges) {
        return None;
    }
    let mut indegree = vec![0usize; node_count];
    let mut adj = vec![Vec::new(); node_count];
    for &(f, t) in edges {
        adj[f].push(t);
        indegree[t] += 1;
    }
    let mut q: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (i, &deg) in indegree.iter().enumerate() {
        if deg == 0 {
            q.push_back(i);
        }
    }
    let mut out = Vec::with_capacity(node_count);
    while let Some(u) = q.pop_front() {
        out.push(u);
        for &v in &adj[u] {
            indegree[v] -= 1;
            if indegree[v] == 0 {
                q.push_back(v);
            }
        }
    }
    if out.len() == node_count {
        Some(out)
    } else {
        None
    }
}

/// Fan-in: more than one edge targeting the same node (invalid per validation rules).
pub fn fan_in_edge_targets(edges: &[(usize, usize)]) -> Vec<usize> {
    let mut count: HashMap<usize, usize> = HashMap::new();
    for &(_, t) in edges {
        *count.entry(t).or_insert(0) += 1;
    }
    count
        .into_iter()
        .filter(|&(_, c)| c > 1)
        .map(|(n, _)| n)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DinDocument, Route, RouteEndpoint};
    use crate::parse::parse_document_json_str;

    const ORCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/orchestrated-scene.din.json"
    ));

    #[test]
    fn orchestrated_scene_route_dag_has_topological_order() {
        let doc: DinDocument = parse_document_json_str(ORCH).expect("fixture");
        let scene = doc.scene_by_id("main").expect("main scene");
        let g = build_scene_route_graph(scene);
        assert!(!directed_graph_has_cycle(g.endpoints.len(), &g.edges));
        let order = topological_order(g.endpoints.len(), &g.edges).expect("dag");
        assert_eq!(order.len(), g.endpoints.len());
    }

    #[test]
    fn two_node_cycle_detected() {
        let scene = Scene {
            id: "s".into(),
            name: "s".into(),
            description: None,
            dsp: vec![],
            inputs: vec![],
            outputs: vec![],
            routes: vec![
                Route {
                    from: RouteEndpoint::SceneInput {
                        input_id: "a".into(),
                    },
                    to: RouteEndpoint::SceneOutput {
                        output_id: "b".into(),
                    },
                    transform: None,
                },
                Route {
                    from: RouteEndpoint::SceneOutput {
                        output_id: "b".into(),
                    },
                    to: RouteEndpoint::SceneInput {
                        input_id: "a".into(),
                    },
                    transform: None,
                },
            ],
            transport: None,
            timeline: None,
            host_bindings: None,
        };
        let g = build_scene_route_graph(&scene);
        assert!(directed_graph_has_cycle(g.endpoints.len(), &g.edges));
        assert!(topological_order(g.endpoints.len(), &g.edges).is_none());
    }
}
