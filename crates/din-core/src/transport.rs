//! Musical transport timing for tempo, bars, beats, and step scheduling.

use crate::utils::{finite_positive_f64, positive_u32};
use serde::{Deserialize, Serialize};

/// Clock source used to advance transport timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Advance from UI frame cadence (`requestAnimationFrame`-style).
    Raf,
    /// Advance from explicit tick deltas.
    #[default]
    Tick,
}

/// Time grid configuration for sequencing and scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Tempo in beats per minute.
    pub bpm: f64,
    /// Numerator of the time signature.
    pub beats_per_bar: u32,
    /// Denominator of the time signature.
    pub beat_unit: u32,
    /// Bars grouped into a phrase.
    pub bars_per_phrase: u32,
    /// Subdivisions per beat.
    pub steps_per_beat: u32,
    /// Swing amount in `[0.0, 0.99]`.
    pub swing: f64,
    /// Runtime update mode.
    pub mode: TransportMode,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            beats_per_bar: 4,
            beat_unit: 4,
            bars_per_phrase: 4,
            steps_per_beat: 4,
            swing: 0.0,
            mode: TransportMode::Tick,
        }
    }
}

/// Computed timing coordinates for one scheduled step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportTick {
    /// Absolute step index since reset.
    pub step_index: u64,
    /// Position of the step inside the current beat.
    pub step_in_beat: u32,
    /// Position of the beat inside the current bar.
    pub beat_in_bar: u32,
    /// Absolute bar index since reset.
    pub bar_index: u64,
    /// Position of the current bar inside the phrase cycle.
    pub phrase_bar: u32,
}

/// Stateful domain object that tracks transport playhead progression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transport {
    config: TransportConfig,
    playing: bool,
    step_index: u64,
    elapsed_step_time: f64,
}

impl Default for Transport {
    fn default() -> Self {
        Self::new(TransportConfig::default())
    }
}

impl Transport {
    /// Creates a transport with sanitized, non-zero configuration values.
    pub fn new(mut config: TransportConfig) -> Self {
        config.bpm = finite_positive_f64(config.bpm, 120.0);
        config.beats_per_bar = positive_u32(config.beats_per_bar, 1);
        config.beat_unit = positive_u32(config.beat_unit, 1);
        config.bars_per_phrase = positive_u32(config.bars_per_phrase, 1);
        config.steps_per_beat = positive_u32(config.steps_per_beat, 1);
        config.swing = config.swing.clamp(0.0, 0.99);

        Self {
            config,
            playing: false,
            step_index: 0,
            elapsed_step_time: 0.0,
        }
    }

    /// Returns the active transport configuration.
    pub fn config(&self) -> TransportConfig {
        self.config
    }

    /// Returns the configured timing mode.
    pub fn mode(&self) -> TransportMode {
        self.config.mode
    }

    /// Returns `true` when the playhead is advancing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Starts playhead progression.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stops playhead progression.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Resets step counters while preserving configuration and play state.
    pub fn reset(&mut self) {
        self.step_index = 0;
        self.elapsed_step_time = 0.0;
    }

    /// Returns the current absolute step index.
    pub fn step_index(&self) -> u64 {
        self.step_index
    }

    /// Returns the duration of one beat in seconds.
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.config.bpm
    }

    /// Returns the duration of one step in seconds.
    pub fn seconds_per_step(&self) -> f64 {
        self.seconds_per_beat() / f64::from(self.config.steps_per_beat)
    }

    /// Advances the playhead and returns every crossed step as [`TransportTick`].
    pub fn advance_seconds(&mut self, delta_seconds: f64) -> Vec<TransportTick> {
        if !self.playing || delta_seconds <= 0.0 {
            return Vec::new();
        }

        let mut ticks = Vec::new();
        self.elapsed_step_time += delta_seconds;
        loop {
            let step_duration = self.current_step_duration_seconds();
            if self.elapsed_step_time < step_duration {
                break;
            }

            self.elapsed_step_time -= step_duration;
            ticks.push(self.current_tick());
            self.step_index = self.step_index.saturating_add(1);
        }

        ticks
    }

    fn current_tick(&self) -> TransportTick {
        let steps_per_beat = u64::from(self.config.steps_per_beat);
        let beats_per_bar = u64::from(self.config.beats_per_bar);
        let bars_per_phrase = u64::from(self.config.bars_per_phrase);

        let step_in_beat = (self.step_index % steps_per_beat) as u32;
        let beat_index = self.step_index / steps_per_beat;
        let beat_in_bar = (beat_index % beats_per_bar) as u32;
        let bar_index = beat_index / beats_per_bar;
        let phrase_bar = (bar_index % bars_per_phrase) as u32;

        TransportTick {
            step_index: self.step_index,
            step_in_beat,
            beat_in_bar,
            bar_index,
            phrase_bar,
        }
    }

    fn current_step_duration_seconds(&self) -> f64 {
        let base = self.seconds_per_step();
        if self.config.swing == 0.0 {
            return base;
        }

        let odd_step = self.step_index % 2 == 1;
        let swing_amount = base * self.config.swing * 0.5;
        if odd_step {
            base + swing_amount
        } else {
            (base - swing_amount).max(base * 0.1)
        }
    }
}
