//! Indexed read-only [`DocumentHandle`] built from an accepted [`crate::ValidationReport`].

use crate::graph::{
    SceneRouteGraph, build_scene_route_graph, directed_graph_has_cycle, fan_in_edge_targets,
    topological_order,
};
use crate::model::{
    Asset, AudioSource, Buffer, BufferView, Collections, DinDocument, DspModule, HostBindings,
    Impulse, MidiSource, SampleSlot, Scene, Timeline, Transport,
};
use crate::report::ValidationReport;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Failed to construct a [`DocumentHandle`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DocumentHandleBuildError {
    /// The validation report rejected the document.
    #[error("document was not accepted by validation")]
    NotAccepted,
}

/// Immutable indexed view over a validated [`DinDocument`].
#[derive(Debug, Clone)]
pub struct DocumentHandle {
    doc: Arc<DinDocument>,
    scene_index: HashMap<String, usize>,
    buffer_index: HashMap<String, usize>,
    buffer_view_index: HashMap<String, usize>,
    audio_source_index: HashMap<String, usize>,
    midi_source_index: HashMap<String, usize>,
    sample_slot_index: HashMap<String, usize>,
    impulse_index: HashMap<String, usize>,
    dsp_module_index: HashMap<String, usize>,
}

impl DocumentHandle {
    /// Builds a handle only when `report.accepted` is true.
    pub fn try_new(
        doc: DinDocument,
        report: &ValidationReport,
    ) -> Result<Self, DocumentHandleBuildError> {
        if !report.accepted {
            return Err(DocumentHandleBuildError::NotAccepted);
        }
        Ok(Self::new_unchecked(Arc::new(doc)))
    }

    /// Builds from an `Arc` after the caller has ensured validation succeeded.
    pub fn from_validated_arc(
        doc: Arc<DinDocument>,
        report: &ValidationReport,
    ) -> Result<Self, DocumentHandleBuildError> {
        if !report.accepted {
            return Err(DocumentHandleBuildError::NotAccepted);
        }
        Ok(Self::new_unchecked(doc))
    }

    fn new_unchecked(doc: Arc<DinDocument>) -> Self {
        let scene_index = doc
            .scenes
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
        let buffer_index = index_by_id(&doc.collections.buffers, |b| b.id.as_str());
        let buffer_view_index = index_by_id(&doc.collections.buffer_views, |b| b.id.as_str());
        let audio_source_index = index_by_id(&doc.collections.audio_sources, |b| b.id.as_str());
        let midi_source_index = index_by_id(&doc.collections.midi_sources, |b| b.id.as_str());
        let sample_slot_index = index_by_id(&doc.collections.sample_slots, |b| b.id.as_str());
        let impulse_index = index_by_id(&doc.collections.impulses, |b| b.id.as_str());
        let dsp_module_index = index_by_id(&doc.collections.dsp_modules, |b| b.id.as_str());
        Self {
            doc,
            scene_index,
            buffer_index,
            buffer_view_index,
            audio_source_index,
            midi_source_index,
            sample_slot_index,
            impulse_index,
            dsp_module_index,
        }
    }

    /// Borrow the full document (immutable).
    pub fn document(&self) -> &DinDocument {
        self.doc.as_ref()
    }

    /// Shared pointer to the document.
    pub fn document_arc(&self) -> &Arc<DinDocument> {
        &self.doc
    }

    /// Default scene per `defaultSceneId` (always present when validation passed).
    pub fn default_scene(&self) -> Option<&Scene> {
        self.scene(&self.doc.default_scene_id)
    }

    /// Resolve a scene by id.
    pub fn scene(&self, id: &str) -> Option<&Scene> {
        let ix = self.scene_index.get(id)?;
        self.doc.scenes.get(*ix)
    }

    /// All scenes in document order.
    pub fn scenes(&self) -> &[Scene] {
        &self.doc.scenes
    }

    /// Asset metadata.
    pub fn asset(&self) -> &Asset {
        &self.doc.asset
    }

    /// Collections block.
    pub fn collections(&self) -> &Collections {
        &self.doc.collections
    }

    /// Buffers.
    pub fn buffers(&self) -> &[Buffer] {
        &self.doc.collections.buffers
    }

    /// Buffer views.
    pub fn buffer_views(&self) -> &[BufferView] {
        &self.doc.collections.buffer_views
    }

    /// Audio sources.
    pub fn audio_sources(&self) -> &[AudioSource] {
        &self.doc.collections.audio_sources
    }

    /// MIDI sources.
    pub fn midi_sources(&self) -> &[MidiSource] {
        &self.doc.collections.midi_sources
    }

    /// Sample slots.
    pub fn sample_slots(&self) -> &[SampleSlot] {
        &self.doc.collections.sample_slots
    }

    /// Impulses.
    pub fn impulses(&self) -> &[Impulse] {
        &self.doc.collections.impulses
    }

    /// DSP module definitions.
    pub fn dsp_modules(&self) -> &[DspModule] {
        &self.doc.collections.dsp_modules
    }

    /// Scene inputs for `scene_id`.
    pub fn scene_inputs(&self, scene_id: &str) -> Option<&[crate::model::PortInput]> {
        Some(self.scene(scene_id)?.inputs.as_slice())
    }

    /// Scene outputs for `scene_id`.
    pub fn scene_outputs(&self, scene_id: &str) -> Option<&[crate::model::PortOutput]> {
        Some(self.scene(scene_id)?.outputs.as_slice())
    }

    /// Routes for `scene_id`.
    pub fn scene_routes(&self, scene_id: &str) -> Option<&[crate::model::Route]> {
        Some(self.scene(scene_id)?.routes.as_slice())
    }

    /// DSP instances for `scene_id`.
    pub fn scene_dsp(&self, scene_id: &str) -> Option<&[crate::model::SceneDsp]> {
        Some(self.scene(scene_id)?.dsp.as_slice())
    }

    /// Transport defaults for `scene_id`.
    pub fn scene_transport(&self, scene_id: &str) -> Option<&Transport> {
        self.scene(scene_id)?.transport.as_ref()
    }

    /// Timeline for `scene_id`.
    pub fn scene_timeline(&self, scene_id: &str) -> Option<&Timeline> {
        self.scene(scene_id)?.timeline.as_ref()
    }

    /// Tracks inside the timeline when present.
    pub fn scene_tracks(&self, scene_id: &str) -> Option<&[crate::model::Track]> {
        Some(self.scene(scene_id)?.timeline.as_ref()?.tracks.as_slice())
    }

    /// Sequencers inside the timeline when present.
    pub fn scene_sequencers(&self, scene_id: &str) -> Option<&[crate::model::Sequencer]> {
        Some(
            self.scene(scene_id)?
                .timeline
                .as_ref()?
                .sequencers
                .as_slice(),
        )
    }

    /// Normalized route graph view for `scene_id`.
    pub fn graph(&self, scene_id: &str) -> Option<SceneGraphView> {
        let scene = self.scene(scene_id)?;
        Some(SceneGraphView::from_scene(scene))
    }

    /// Lookup helpers (O(1)) by stable collection id.
    pub fn buffer_by_id(&self, id: &str) -> Option<&Buffer> {
        let ix = *self.buffer_index.get(id)?;
        self.doc.collections.buffers.get(ix)
    }

    /// Lookup buffer view by id.
    pub fn buffer_view_by_id(&self, id: &str) -> Option<&BufferView> {
        let ix = *self.buffer_view_index.get(id)?;
        self.doc.collections.buffer_views.get(ix)
    }

    /// Lookup audio source by id.
    pub fn audio_source_by_id(&self, id: &str) -> Option<&AudioSource> {
        let ix = *self.audio_source_index.get(id)?;
        self.doc.collections.audio_sources.get(ix)
    }

    /// Lookup MIDI source by id.
    pub fn midi_source_by_id(&self, id: &str) -> Option<&MidiSource> {
        let ix = *self.midi_source_index.get(id)?;
        self.doc.collections.midi_sources.get(ix)
    }

    /// Lookup DSP module definition by id.
    pub fn dsp_module_by_id(&self, id: &str) -> Option<&DspModule> {
        let ix = *self.dsp_module_index.get(id)?;
        self.doc.collections.dsp_modules.get(ix)
    }

    /// Lookup sample slot by id.
    pub fn sample_slot_by_id(&self, id: &str) -> Option<&SampleSlot> {
        let ix = *self.sample_slot_index.get(id)?;
        self.doc.collections.sample_slots.get(ix)
    }

    /// Lookup impulse by id.
    pub fn impulse_by_id(&self, id: &str) -> Option<&Impulse> {
        let ix = *self.impulse_index.get(id)?;
        self.doc.collections.impulses.get(ix)
    }

    /// Host bindings for a scene when present.
    pub fn scene_host_bindings(&self, scene_id: &str) -> Option<&HostBindings> {
        self.scene(scene_id)?.host_bindings.as_ref()
    }
}

fn index_by_id<T>(items: &[T], id: impl Fn(&T) -> &str) -> HashMap<String, usize> {
    items
        .iter()
        .enumerate()
        .map(|(i, t)| (id(t).to_string(), i))
        .collect()
}

/// Inspection-friendly route graph for one scene.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneGraphView {
    /// Underlying route digraph.
    pub route_graph: SceneRouteGraph,
    /// Topological order of endpoint indices when acyclic.
    pub topological_order: Option<Vec<usize>>,
    /// True when the route graph contains a directed cycle.
    pub has_cycle: bool,
}

impl SceneGraphView {
    /// Builds a graph view and summary flags from a scene.
    pub fn from_scene(scene: &Scene) -> Self {
        let route_graph = build_scene_route_graph(scene);
        let n = route_graph.endpoints.len();
        let has_cycle = directed_graph_has_cycle(n, &route_graph.edges);
        let topological_order = if has_cycle {
            None
        } else {
            topological_order(n, &route_graph.edges)
        };
        Self {
            route_graph,
            topological_order,
            has_cycle,
        }
    }

    /// Endpoints that receive more than one incoming route edge (invalid fan-in).
    pub fn fan_in_targets(&self) -> Vec<usize> {
        fan_in_edge_targets(&self.route_graph.edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_document_json_str;
    use crate::validate::validate_document;

    const MINIMAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/minimal.din.json"
    ));

    #[test]
    fn handle_requires_accepted_report() {
        let doc = parse_document_json_str(MINIMAL).expect("parse");
        let bad = ValidationReport::reject(vec![]);
        assert!(DocumentHandle::try_new(doc.clone(), &bad).is_err());
    }

    #[test]
    fn default_scene_matches_default_scene_id() {
        let doc = parse_document_json_str(MINIMAL).expect("parse");
        let report = validate_document(&doc);
        let handle = DocumentHandle::try_new(doc, &report).expect("handle");
        let def = handle.default_scene().expect("default");
        assert_eq!(def.id, handle.document().default_scene_id);
    }
}
