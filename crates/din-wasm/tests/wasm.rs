use din_wasm::{compile_patch_impl, migrate_patch_impl, patch_interface_impl, validate_patch_impl};

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

#[test]
fn wasm_helpers_reuse_shared_patch_logic() {
    assert!(validate_patch_impl(FIXTURE).expect("validation should succeed"));

    let migrated = migrate_patch_impl(FIXTURE).expect("migration should succeed");
    assert!(migrated.contains("\"cutoff\""));

    let interface = patch_interface_impl(FIXTURE).expect("interface extraction should succeed");
    assert_eq!(interface.inputs.len(), 1);
    assert_eq!(interface.events.len(), 1);

    let compiled = compile_patch_impl(FIXTURE).expect("compile should succeed");
    assert_eq!(compiled.audio_connections.len(), 5);
    assert_eq!(compiled.transport_connections.len(), 1);
}
