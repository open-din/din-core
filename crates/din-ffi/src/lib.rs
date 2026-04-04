use din_core::{CompiledGraph, CoreError, Engine, EngineConfig, Graph, MidiMessage, PatchImporter};
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

pub struct DinGraphHandle {
    graph: Graph,
    compiled: CompiledGraph,
}

pub struct DinEngineHandle {
    engine: Engine,
}

/// Frees a C string previously returned by this library (for example [`din_graph_interface_json`]).
///
/// # Safety
/// `value` must be null or a pointer returned by this FFI that has not been freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn din_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn din_patch_validate_json(
    json: *const c_char,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let json = c_string_arg(json)?;
        let patch = PatchImporter::from_json(&json)?;
        din_core::validate_patch_document(&patch)?;
        Ok(true)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn din_patch_migrate_json(
    json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    with_error_slot(error_out, || {
        let json = c_string_arg(json)?;
        let patch = PatchImporter::from_json(&json)?;
        let migrated = din_core::PatchExporter::to_json(&patch)?;
        Ok(into_c_string(migrated))
    })
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn din_graph_create_from_patch_json(
    json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    with_error_slot(error_out, || {
        let json = c_string_arg(json)?;
        let patch = PatchImporter::from_json(&json)?;
        let graph = Graph::from_patch(&patch)?;
        let compiled = graph.compile();
        let handle = DinGraphHandle { graph, compiled };
        Ok(Box::into_raw(Box::new(handle)) as *mut c_void)
    })
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn din_graph_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(handle as *mut DinGraphHandle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn din_graph_interface_json(
    handle: *const c_void,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    with_error_slot(error_out, || {
        let handle = graph_handle_ref(handle)?;
        let payload = serde_json::to_string_pretty(handle.graph.interface())?;
        Ok(into_c_string(payload))
    })
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_create(
    graph_handle: *const c_void,
    sample_rate: f32,
    channels: usize,
    block_size: usize,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    with_error_slot(error_out, || {
        let graph_handle = graph_handle_ref(graph_handle)?;
        let engine = Engine::new(
            graph_handle.compiled.clone(),
            EngineConfig {
                sample_rate,
                channels,
                block_size,
            },
        );
        Ok(Box::into_raw(Box::new(DinEngineHandle { engine })) as *mut c_void)
    })
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(handle as *mut DinEngineHandle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_set_input(
    handle: *mut c_void,
    key: *const c_char,
    value: f32,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let engine = engine_handle_mut(handle)?;
        let key = c_string_arg(key)?;
        engine.engine.set_input(&key, value)?;
        Ok(true)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_trigger_event(
    handle: *mut c_void,
    key: *const c_char,
    token: u64,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let engine = engine_handle_mut(handle)?;
        let key = c_string_arg(key)?;
        engine.engine.trigger_event(&key, token)?;
        Ok(true)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_push_midi(
    handle: *mut c_void,
    status: u8,
    data1: u8,
    data2: u8,
    frame_offset: u32,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let engine = engine_handle_mut(handle)?;
        engine.engine.push_midi(MidiMessage {
            status,
            data1,
            data2,
            frame_offset,
        });
        Ok(true)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_load_asset(
    handle: *mut c_void,
    path: *const c_char,
    data: *const u8,
    len: usize,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let engine = engine_handle_mut(handle)?;
        let path = c_string_arg(path)?;
        let bytes = slice_arg(data, len)?.to_vec();
        engine.engine.load_asset(path, bytes);
        Ok(true)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn din_engine_render(
    handle: *mut c_void,
    out_buffer: *mut f32,
    len: usize,
    error_out: *mut *mut c_char,
) -> bool {
    with_error_slot(error_out, || {
        let engine = engine_handle_mut(handle)?;
        let rendered = engine.engine.render_block();
        if rendered.len() != len {
            return Err(CoreError::Patch(din_core::PatchError::Invalid(format!(
                "expected output buffer length {}, got {len}",
                rendered.len()
            ))));
        }

        let out = mut_slice_arg(out_buffer, len)?;
        out.copy_from_slice(&rendered);
        Ok(true)
    })
    .unwrap_or(false)
}

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .expect("CString conversion should not fail for serialized JSON")
        .into_raw()
}

fn c_string_arg(ptr: *const c_char) -> Result<String, CoreError> {
    if ptr.is_null() {
        return Err(CoreError::Patch(din_core::PatchError::Invalid(
            "expected a non-null string pointer".to_string(),
        )));
    }

    let c_str = unsafe { CStr::from_ptr(ptr) };
    Ok(c_str.to_string_lossy().into_owned())
}

fn graph_handle_ref(handle: *const c_void) -> Result<&'static DinGraphHandle, CoreError> {
    if handle.is_null() {
        return Err(CoreError::Patch(din_core::PatchError::Invalid(
            "expected a non-null graph handle".to_string(),
        )));
    }

    Ok(unsafe { &*(handle as *const DinGraphHandle) })
}

fn engine_handle_mut(handle: *mut c_void) -> Result<&'static mut DinEngineHandle, CoreError> {
    if handle.is_null() {
        return Err(CoreError::Patch(din_core::PatchError::Invalid(
            "expected a non-null engine handle".to_string(),
        )));
    }

    Ok(unsafe { &mut *(handle as *mut DinEngineHandle) })
}

fn slice_arg<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], CoreError> {
    if ptr.is_null() {
        return Err(CoreError::Patch(din_core::PatchError::Invalid(
            "expected a non-null byte pointer".to_string(),
        )));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn mut_slice_arg<'a>(ptr: *mut f32, len: usize) -> Result<&'a mut [f32], CoreError> {
    if ptr.is_null() {
        return Err(CoreError::Patch(din_core::PatchError::Invalid(
            "expected a non-null output buffer".to_string(),
        )));
    }
    Ok(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
}

fn with_error_slot<T>(
    error_out: *mut *mut c_char,
    f: impl FnOnce() -> Result<T, CoreError>,
) -> Option<T> {
    if !error_out.is_null() {
        unsafe {
            *error_out = ptr::null_mut();
        }
    }

    match f() {
        Ok(value) => Some(value),
        Err(error) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = into_c_string(error.to_string());
                }
            }
            None
        }
    }
}
