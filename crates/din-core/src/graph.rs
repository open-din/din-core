use crate::registry::{NodeRegistryEntry, node_registry};
use din_patch::{
    NodeKind, PatchConnection, PatchDocument, PatchInterface, PatchNode, get_transport_connections,
    is_audio_connection_like, migrate_patch_document,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionKind {
    Audio,
    Transport,
    TriggerGate,
    Control,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphConnection {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
    pub kind: ConnectionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub patch: PatchDocument,
    pub nodes: Vec<PatchNode>,
    pub connections: Vec<GraphConnection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledGraph {
    pub graph: Graph,
    pub audio_connections: Vec<GraphConnection>,
    pub transport_connections: Vec<GraphConnection>,
    pub trigger_connections: Vec<GraphConnection>,
    pub control_connections: Vec<GraphConnection>,
    pub transport_connected_ids: Vec<String>,
}

impl Graph {
    pub fn from_patch(patch: &PatchDocument) -> Result<Self, CoreError> {
        let patch = migrate_patch_document(patch)?;
        let node_by_id = patch
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        let transport_connected_ids = get_transport_connections(&patch.connections, &node_by_id);

        let connections = patch
            .connections
            .iter()
            .map(|connection| GraphConnection {
                id: connection.id.clone(),
                source: connection.source.clone(),
                target: connection.target.clone(),
                source_handle: connection.source_handle.clone(),
                target_handle: connection.target_handle.clone(),
                kind: classify_connection(connection, &node_by_id, &transport_connected_ids),
            })
            .collect::<Vec<_>>();

        Ok(Self {
            nodes: patch.nodes.clone(),
            patch,
            connections,
        })
    }

    pub fn interface(&self) -> &PatchInterface {
        &self.patch.interface
    }

    pub fn node(&self, id: &str) -> Option<&PatchNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn registry_entry(&self, kind: NodeKind) -> Option<&'static NodeRegistryEntry> {
        node_registry().iter().find(|entry| entry.kind == kind)
    }

    pub fn compile(&self) -> CompiledGraph {
        let audio_connections = self
            .connections
            .iter()
            .filter(|connection| connection.kind == ConnectionKind::Audio)
            .cloned()
            .collect::<Vec<_>>();
        let transport_connections = self
            .connections
            .iter()
            .filter(|connection| connection.kind == ConnectionKind::Transport)
            .cloned()
            .collect::<Vec<_>>();
        let trigger_connections = self
            .connections
            .iter()
            .filter(|connection| connection.kind == ConnectionKind::TriggerGate)
            .cloned()
            .collect::<Vec<_>>();
        let control_connections = self
            .connections
            .iter()
            .filter(|connection| connection.kind == ConnectionKind::Control)
            .cloned()
            .collect::<Vec<_>>();
        let transport_connected_ids = transport_connections
            .iter()
            .map(|connection| connection.target.clone())
            .collect::<Vec<_>>();

        CompiledGraph {
            graph: self.clone(),
            audio_connections,
            transport_connections,
            trigger_connections,
            control_connections,
            transport_connected_ids,
        }
    }
}

impl CompiledGraph {
    pub fn from_patch(patch: &PatchDocument) -> Result<Self, CoreError> {
        Ok(Graph::from_patch(patch)?.compile())
    }

    pub fn interface(&self) -> &PatchInterface {
        self.graph.interface()
    }
}

fn classify_connection(
    connection: &PatchConnection,
    node_by_id: &BTreeMap<String, &PatchNode>,
    transport_connected_ids: &std::collections::BTreeSet<String>,
) -> ConnectionKind {
    if is_audio_connection_like(connection, node_by_id) {
        return ConnectionKind::Audio;
    }

    if transport_connected_ids.contains(&connection.target)
        && connection.source_handle.as_deref() == Some("out")
        && connection.target_handle.as_deref() == Some("transport")
    {
        return ConnectionKind::Transport;
    }

    if matches!(
        connection.target_handle.as_deref(),
        Some("trigger" | "gate")
    ) {
        return ConnectionKind::TriggerGate;
    }

    ConnectionKind::Control
}
