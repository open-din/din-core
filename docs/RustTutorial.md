# Rust Tutorial: Using `din-core`

This guide shows a practical host-side flow for `din-core`:

1. Parse patch JSON
2. Build and compile a graph
3. Create an engine and render audio blocks
4. Drive interface inputs, events, and MIDI
5. Use transport and helper utilities

## 1) Add dependencies

In your crate:

```toml
[dependencies]
din-core = { path = "../din-core/crates/din-core" }
serde_json = "1"
```

If you use this from outside the workspace, replace the local path with a version from crates.io once published.

## 2) Parse and validate a patch

`PatchImporter::from_json` parses and migrates the patch document.

```rust
use din_core::{PatchImporter, PatchDocument};

fn load_patch(json: &str) -> Result<PatchDocument, Box<dyn std::error::Error>> {
    let patch = PatchImporter::from_json(json)?;
    Ok(patch)
}
```

You can read JSON from disk:

```rust
let json = std::fs::read_to_string("fixtures/canonical_patch.json")?;
let patch = load_patch(&json)?;
println!("loaded patch: {} ({} nodes)", patch.name, patch.nodes.len());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 3) Compile graph + create engine

The runtime consumes a `CompiledGraph` plus an `EngineConfig`.

```rust
use din_core::{CompiledGraph, Engine, EngineConfig, PatchImporter};

fn make_engine(json: &str) -> Result<Engine, Box<dyn std::error::Error>> {
    let patch = PatchImporter::from_json(json)?;
    let compiled = CompiledGraph::from_patch(&patch)?;
    let engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 2,
            block_size: 128,
        },
    )?;
    Ok(engine)
}
```

## 4) Render an audio block

`render_block()` returns interleaved samples with length `block_size * channels`.

```rust
let mut engine = make_engine(&std::fs::read_to_string("fixtures/canonical_patch.json")?)?;
let block = engine.render_block();
println!("rendered {} samples", block.len());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 5) Set interface inputs, trigger events, push MIDI

Before rendering a block, update control state:

```rust
use din_core::MidiMessage;

engine.set_input("cutoff", 0.7)?;
engine.trigger_event("bang", 1)?;
engine.push_midi(MidiMessage {
    status: 0x90, // Note On, channel 1
    data1: 60,    // C4
    data2: 100,   // velocity
    frame_offset: 0,
});

let block = engine.render_block();
assert!(block.iter().any(|sample| sample.abs() > 0.000_1));
# Ok::<(), Box<dyn std::error::Error>>(())
```

If you pass an unknown input/event key, `din-core` returns `CoreError::UnknownInputKey` or `CoreError::UnknownEventKey`.

## 6) Handle `patch` node limitation in native v1

Native runtime v1 intentionally rejects graphs containing `NodeKind::Patch`:

```rust
use din_core::{CompiledGraph, Engine, EngineConfig, PatchImporter};

let json = std::fs::read_to_string("fixtures/canonical_patch.json")?;
let patch = PatchImporter::from_json(&json)?;
let compiled = CompiledGraph::from_patch(&patch)?;
let result = Engine::new(compiled, EngineConfig::default());

if let Err(err) = result {
    eprintln!("engine init failed: {err}");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use this behavior as a guardrail while native nested patch execution is not yet implemented.

## 7) Export normalized JSON

Use `PatchExporter` to emit migrated, pretty-printed JSON:

```rust
use din_core::{PatchExporter, PatchImporter};

let json = std::fs::read_to_string("fixtures/canonical_patch.json")?;
let patch = PatchImporter::from_json(&json)?;
let normalized = PatchExporter::to_json(&patch)?;
std::fs::write("normalized.patch.json", normalized)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 8) Transport timing

`Transport` is useful for host scheduling and sequencer clocks:

```rust
use din_core::{Transport, TransportConfig, TransportMode};

let mut transport = Transport::new(TransportConfig {
    mode: TransportMode::Tick,
    ..TransportConfig::default()
});
transport.play();

let step_seconds = transport.seconds_per_step();
let ticks = transport.advance_seconds(step_seconds * 2.2);
for tick in ticks {
    println!(
        "step={} beat_in_bar={} bar={}",
        tick.step_index, tick.beat_in_bar, tick.bar_index
    );
}
```

## 9) Utility helpers (notes + scalar nodes)

`din-core` also exports note and scalar helpers:

```rust
use din_core::{
    ClampMode, CompareOperation, MathOperation, clamp, compare, math, midi_to_note, mix, note_to_freq,
    switch_value,
};

let hz = note_to_freq("La4").unwrap();
let note = midi_to_note(61, true); // Db4
let sum = math(MathOperation::Add, 2.0, 3.0, 0.0);
let is_gt = compare(CompareOperation::GreaterThan, 4.0, 3.0);
let lerp = mix(0.0, 10.0, 0.25, true);
let clipped = clamp(12.0, 0.0, 10.0, ClampMode::Clamp);
let switched = switch_value(1, &[4.0, 9.0, 16.0]);

println!("{hz} {note} {sum} {is_gt} {lerp} {clipped} {switched}");
```

## End-to-end sample (`main.rs`)

This sample removes unsupported `patch` nodes from the canonical fixture, then renders one block:

```rust
use din_core::{CompiledGraph, Engine, EngineConfig, MidiMessage, PatchImporter};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string("fixtures/canonical_patch.json")?;
    let mut patch: Value = serde_json::from_str(&json)?;

    patch["nodes"] = Value::Array(
        patch["nodes"]
            .as_array()
            .ok_or("nodes must be array")?
            .iter()
            .filter(|node| node["id"] != "patch-1")
            .cloned()
            .collect(),
    );
    patch["connections"] = Value::Array(
        patch["connections"]
            .as_array()
            .ok_or("connections must be array")?
            .iter()
            .filter(|connection| {
                connection["source"] != "patch-1" && connection["target"] != "patch-1"
            })
            .cloned()
            .collect(),
    );

    let patch_json = serde_json::to_string(&patch)?;
    let patch = PatchImporter::from_json(&patch_json)?;
    let compiled = CompiledGraph::from_patch(&patch)?;
    let mut engine = Engine::new(compiled, EngineConfig::default())?;

    let _ = engine.set_input("cutoff", 0.7);
    let _ = engine.trigger_event("bang", 1);
    engine.push_midi(MidiMessage {
        status: 0x90,
        data1: 60,
        data2: 100,
        frame_offset: 0,
    });

    let block = engine.render_block();
    println!(
        "rendered block: samples={}, non_zero={}",
        block.len(),
        block.iter().any(|s| s.abs() > 0.000_1)
    );

    Ok(())
}
```
