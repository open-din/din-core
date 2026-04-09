//! Clean audio-facing namespace for engine and DSP helpers.

/// Data and scalar operators used by audio/control nodes.
pub use crate::data::{
    ClampMode, CompareOperation, MathOperation, clamp, compare, math, mix, switch_value,
};
/// Conservative native runtime and event/midi domain types.
pub use crate::engine::{Engine, EngineConfig, MidiMessage, TriggerEvent};
/// Music theory and note parsing helpers.
pub use crate::notes::{
    FrenchNoteName, NoteName, ParsedNote, midi_to_freq, midi_to_note, note_from_french,
    note_to_french, note_to_freq, note_to_midi, parse_note,
};
