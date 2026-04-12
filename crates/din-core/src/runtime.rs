//! DinDocument runtime session and transport / sequencer / bridge controllers (v2 surface).
//!
//! Typed document model, parse, validation, handle, and graph live in [`din_document`].

use din_document::DocumentHandle;
use std::sync::Arc;
use thiserror::Error;

/// Failed to create a [`RuntimeSession`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuntimeSessionError {
    /// Scene id is not present on the document handle.
    #[error("unknown scene id {0:?}")]
    UnknownScene(String),
}

/// Mutable runtime state for one selected scene over an immutable [`DocumentHandle`].
#[derive(Debug)]
pub struct RuntimeSession {
    handle: Arc<DocumentHandle>,
    scene_id: String,
    transport: TransportController,
    sequencer: SequencerController,
    bridge: BridgeController,
}

impl RuntimeSession {
    /// Creates a session; fails when `scene_id` is not found on the handle.
    pub fn new(handle: Arc<DocumentHandle>, scene_id: &str) -> Result<Self, RuntimeSessionError> {
        if handle.scene(scene_id).is_none() {
            return Err(RuntimeSessionError::UnknownScene(scene_id.to_string()));
        }
        Ok(Self {
            handle,
            scene_id: scene_id.to_string(),
            transport: TransportController::default(),
            sequencer: SequencerController::default(),
            bridge: BridgeController::default(),
        })
    }

    /// Selected scene id.
    pub fn scene_id(&self) -> &str {
        &self.scene_id
    }

    /// Immutable document handle (includes the [`DinDocument`](din_document::DinDocument)).
    pub fn document_handle(&self) -> &DocumentHandle {
        self.handle.as_ref()
    }

    /// Shared handle pointer.
    pub fn document_handle_arc(&self) -> &Arc<DocumentHandle> {
        &self.handle
    }

    /// Transport controller.
    pub fn transport(&self) -> &TransportController {
        &self.transport
    }

    /// Mutable transport controller.
    pub fn transport_mut(&mut self) -> &mut TransportController {
        &mut self.transport
    }

    /// Sequencer controller.
    pub fn sequencer(&self) -> &SequencerController {
        &self.sequencer
    }

    /// Mutable sequencer controller.
    pub fn sequencer_mut(&mut self) -> &mut SequencerController {
        &mut self.sequencer
    }

    /// Bridge controller.
    pub fn bridge(&self) -> &BridgeController {
        &self.bridge
    }

    /// Mutable bridge controller.
    pub fn bridge_mut(&mut self) -> &mut BridgeController {
        &mut self.bridge
    }
}

/// Transport control state (scalar-only hot path; no per-tick allocation).
#[derive(Debug, Clone)]
pub struct TransportController {
    playing: bool,
    paused: bool,
    bpm: f64,
    seek_beats: f64,
}

impl Default for TransportController {
    fn default() -> Self {
        Self {
            playing: false,
            paused: false,
            bpm: 120.0,
            seek_beats: 0.0,
        }
    }
}

/// Transport commands applied to [`TransportController`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportCommand {
    /// Start or resume playback semantics.
    Start,
    /// Stop playback and reset position.
    Stop,
    /// Pause.
    Pause,
    /// Resume from pause.
    Resume,
    /// Seek to a musical time in beats.
    Seek(f64),
}

impl TransportController {
    /// Current playing flag.
    pub fn playing(&self) -> bool {
        self.playing
    }

    /// Current paused flag.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Tempo in BPM.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Set tempo (BPM).
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    /// Seek position in beats (last seek target).
    pub fn seek_beats(&self) -> f64 {
        self.seek_beats
    }

    /// Apply a transport command.
    pub fn apply(&mut self, cmd: TransportCommand) {
        match cmd {
            TransportCommand::Start => {
                self.playing = true;
                self.paused = false;
            }
            TransportCommand::Stop => {
                self.playing = false;
                self.paused = false;
                self.seek_beats = 0.0;
            }
            TransportCommand::Pause => {
                if self.playing {
                    self.paused = true;
                }
            }
            TransportCommand::Resume => {
                if self.paused {
                    self.paused = false;
                    self.playing = true;
                }
            }
            TransportCommand::Seek(beats) => {
                self.seek_beats = beats;
            }
        }
    }

    /// Worker/audio-thread safe tick: updates only inline state (no allocation).
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        if self.playing {
            self.seek_beats += 1.0 / 128.0;
        }
    }
}

/// Sequencer runtime control (minimal deterministic state).
#[derive(Debug, Default)]
pub struct SequencerController {
    generation: u64,
    last_sequencer_id: Option<String>,
}

impl SequencerController {
    /// Monotonic generation counter for test assertions.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Last sequencer id touched by trigger APIs.
    pub fn last_sequencer_id(&self) -> Option<&str> {
        self.last_sequencer_id.as_deref()
    }

    /// Trigger a sequencer instance (id is copied once per command, not per tick).
    pub fn trigger(&mut self, sequencer_id: &str) {
        self.generation = self.generation.wrapping_add(1);
        self.last_sequencer_id = Some(sequencer_id.to_string());
    }

    /// Retrigger (same as trigger for this skeleton).
    pub fn retrigger(&mut self, sequencer_id: &str) {
        self.trigger(sequencer_id);
    }

    /// Stop tracking (clears last id).
    pub fn stop(&mut self) {
        self.last_sequencer_id = None;
    }

    /// Hot path: no heap traffic.
    pub fn tick(&mut self) {}
}

/// Signal bridge entry points (placeholder).
#[derive(Debug, Default)]
pub struct BridgeController {}

#[cfg(test)]
mod tests {
    use super::*;
    use din_document::{DocumentHandle, validate_document};
    use std::sync::Arc;

    const MINIMAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/minimal.din.json"
    ));

    #[test]
    fn session_rejects_unknown_scene() {
        let doc = din_document::parse_document_json_str(MINIMAL).expect("parse");
        let report = validate_document(&doc);
        let handle = Arc::new(DocumentHandle::try_new(doc, &report).expect("handle"));
        let err = RuntimeSession::new(handle, "nope").expect_err("unknown scene");
        assert!(matches!(err, RuntimeSessionError::UnknownScene(_)));
    }

    #[test]
    fn transport_commands_and_tick_no_panic() {
        let mut t = TransportController::default();
        t.apply(TransportCommand::Start);
        for _ in 0..10_000 {
            t.tick();
        }
        assert!(t.playing());
    }
}
