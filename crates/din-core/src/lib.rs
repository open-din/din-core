//! Native runtime for DinDocument v2: re-exports the document stack and session controllers.
//!
//! Order follows the v2 feature slice: typed model → parse → validation → handle/query → graph
//! (via [`din_document`]), then runtime session and transport / sequencer / bridge.
//!
//! Legacy react-din patch JSON, DSP audio engines, and exhaustive node registries are out of scope.

pub use din_document::*;

pub mod runtime;

pub use runtime::{
    BridgeController, RuntimeSession, RuntimeSessionError, SequencerController, TransportCommand,
    TransportController,
};

/// Version string for this `din-core` crate, re-exported for thin wrapper bindings.
pub const DIN_CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
