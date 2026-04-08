//! Load `fixtures/canonical_patch.json`, validate it, and prove serde round-trip parity.
//!
//! Run from the workspace root:
//! `cargo run -p din-patch --example parse_canonical_patch`

use din_patch::{PatchError, parse_patch_document, validate_patch_document};

fn main() -> Result<(), PatchError> {
    let fixture = include_str!("../../../fixtures/canonical_patch.json");
    let patch = parse_patch_document(fixture)?;
    validate_patch_document(&patch)?;

    let round_trip = serde_json::to_string(&patch).map_err(PatchError::Json)?;
    let again = parse_patch_document(&round_trip)?;
    validate_patch_document(&again)?;

    if patch != again {
        return Err(PatchError::Invalid(
            "serde round-trip produced a different PatchDocument".into(),
        ));
    }

    eprintln!(
        "OK: patch {:?} — {} nodes, {} connections",
        patch.name,
        patch.nodes.len(),
        patch.connections.len()
    );

    Ok(())
}
