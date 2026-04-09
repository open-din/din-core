//! Note-name parsing and Western / French conversions shared by tooling and tests.

use serde::{Deserialize, Serialize};

/// Canonical Western chromatic pitch classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteName {
    /// C natural.
    C,
    /// C sharp.
    Cs,
    /// D natural.
    D,
    /// D sharp.
    Ds,
    /// E natural.
    E,
    /// F natural.
    F,
    /// F sharp.
    Fs,
    /// G natural.
    G,
    /// G sharp.
    Gs,
    /// A natural.
    A,
    /// A sharp.
    As,
    /// B natural.
    B,
}

/// Canonical French/solfege note classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrenchNoteName {
    /// Do.
    Do,
    /// Re.
    Re,
    /// Mi.
    Mi,
    /// Fa.
    Fa,
    /// Sol.
    Sol,
    /// La.
    La,
    /// Si.
    Si,
}

/// Parsed note descriptor with normalized naming and pitch values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedNote {
    /// Normalized note name (`C`, `Db`, `F#`, ...).
    pub note: String,
    /// Parsed octave index.
    pub octave: i8,
    /// MIDI note number in `[0, 127]`.
    pub midi: u8,
    /// Frequency in hertz.
    pub frequency: f64,
}

/// Parses note strings like `A4`, `Db3`, or `Sol-1`.
///
/// # Examples
///
/// ```
/// use din_core::parse_note;
///
/// let parsed = parse_note("A4").expect("A4 should parse");
/// assert_eq!(parsed.midi, 69);
/// ```
pub fn parse_note(input: &str) -> Option<ParsedNote> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let split_at = trimmed
        .find(|ch: char| ch == '-' || ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (name_part, octave_part) = trimmed.split_at(split_at);
    let octave = octave_part.parse::<i8>().ok()?;
    let normalized = normalize_note_name(name_part)?;
    let midi = note_name_to_midi(&normalized, octave)?;

    Some(ParsedNote {
        note: normalized,
        octave,
        midi,
        frequency: midi_to_freq(midi),
    })
}

/// Converts a note string to MIDI note number.
pub fn note_to_midi(input: &str) -> Option<u8> {
    parse_note(input).map(|parsed| parsed.midi)
}

/// Converts a MIDI note number to a note string like `C4`.
///
/// Set `prefer_flats` to use flat spellings (`Db`) over sharps (`C#`).
pub fn midi_to_note(midi: u8, prefer_flats: bool) -> String {
    let note_names_sharp = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let note_names_flat = [
        "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
    ];
    let table = if prefer_flats {
        &note_names_flat
    } else {
        &note_names_sharp
    };
    let note = table[(midi % 12) as usize];
    let octave = (midi / 12) as i16 - 1;
    format!("{note}{octave}")
}

/// Converts a MIDI note number to frequency in hertz (A4 = 440 Hz).
///
/// # Examples
///
/// ```
/// use din_core::midi_to_freq;
///
/// assert!((midi_to_freq(69) - 440.0).abs() < f64::EPSILON);
/// ```
pub fn midi_to_freq(midi: u8) -> f64 {
    440.0 * 2.0f64.powf((f64::from(midi) - 69.0) / 12.0)
}

/// Converts a note string directly to frequency in hertz.
pub fn note_to_freq(input: &str) -> Option<f64> {
    note_to_midi(input).map(midi_to_freq)
}

/// Converts a Western note name to French spelling.
pub fn note_to_french(input: &str) -> Option<String> {
    let normalized = normalize_note_name(input)?;
    let french = match normalized.as_str() {
        "C" => "Do",
        "C#" => "Do#",
        "Db" => "Reb",
        "D" => "Re",
        "D#" => "Re#",
        "Eb" => "Mib",
        "E" => "Mi",
        "F" => "Fa",
        "F#" => "Fa#",
        "Gb" => "Solb",
        "G" => "Sol",
        "G#" => "Sol#",
        "Ab" => "Lab",
        "A" => "La",
        "A#" => "La#",
        "Bb" => "Sib",
        "B" => "Si",
        _ => return None,
    };
    Some(french.to_string())
}

/// Converts a French note spelling to normalized Western spelling.
pub fn note_from_french(input: &str) -> Option<String> {
    normalize_note_name(input)
}

fn normalize_note_name(input: &str) -> Option<String> {
    let normalized = input.trim().replace('♯', "#").replace('♭', "b");
    let lower = normalized.to_ascii_lowercase();

    let mapped = match lower.as_str() {
        "c" => "C",
        "c#" | "db" => {
            if lower.ends_with('b') {
                "Db"
            } else {
                "C#"
            }
        }
        "d" => "D",
        "d#" | "eb" => {
            if lower.ends_with('b') {
                "Eb"
            } else {
                "D#"
            }
        }
        "e" => "E",
        "f" => "F",
        "f#" | "gb" => {
            if lower.ends_with('b') {
                "Gb"
            } else {
                "F#"
            }
        }
        "g" => "G",
        "g#" | "ab" => {
            if lower.ends_with('b') {
                "Ab"
            } else {
                "G#"
            }
        }
        "a" => "A",
        "a#" | "bb" => {
            if lower.ends_with('b') {
                "Bb"
            } else {
                "A#"
            }
        }
        "b" => "B",
        "do" => "C",
        "do#" | "reb" => {
            if lower.ends_with('b') {
                "Db"
            } else {
                "C#"
            }
        }
        "re" => "D",
        "re#" | "mib" => {
            if lower.ends_with('b') {
                "Eb"
            } else {
                "D#"
            }
        }
        "mi" => "E",
        "fa" => "F",
        "fa#" | "solb" => {
            if lower.ends_with('b') {
                "Gb"
            } else {
                "F#"
            }
        }
        "sol" => "G",
        "sol#" | "lab" => {
            if lower.ends_with('b') {
                "Ab"
            } else {
                "G#"
            }
        }
        "la" => "A",
        "la#" | "sib" => {
            if lower.ends_with('b') {
                "Bb"
            } else {
                "A#"
            }
        }
        "si" => "B",
        _ => return None,
    };

    Some(mapped.to_string())
}

fn note_name_to_midi(name: &str, octave: i8) -> Option<u8> {
    let pitch_class = match name {
        "C" => 0,
        "C#" | "Db" => 1,
        "D" => 2,
        "D#" | "Eb" => 3,
        "E" => 4,
        "F" => 5,
        "F#" | "Gb" => 6,
        "G" => 7,
        "G#" | "Ab" => 8,
        "A" => 9,
        "A#" | "Bb" => 10,
        "B" => 11,
        _ => return None,
    };
    let value = (i16::from(octave) + 1) * 12 + pitch_class;
    u8::try_from(value).ok()
}
