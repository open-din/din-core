# TypeScript Tutorial: Using `din-wasm`

This tutorial shows how to use `din-core` from TypeScript via the `din-wasm` package.

You will learn how to:

1. Initialize the WASM module
2. Validate and migrate patch JSON
3. Inspect interfaces and compiled graph summaries
4. Render audio blocks
5. Drive runtime controls (inputs, events, MIDI)
6. Use transport and helper utilities

## 1) Install and initialize

`din-core` is Rust-first. The TypeScript API is exposed through `din-wasm`.

```bash
npm install din-wasm
```

Then initialize once at app startup:

```ts
import init from "din-wasm";

await init();
```

> All exported functions throw on failure. Wrap calls in `try/catch` at boundaries.

## 2) Validate and migrate patch JSON

```ts
import init, { migrate_patch, validate_patch } from "din-wasm";

await init();

const patchJson = await Bun.file("fixtures/canonical_patch.json").text();
// or: const patchJson = await fs.promises.readFile("fixtures/canonical_patch.json", "utf8");

const isValid = validate_patch(patchJson);
console.log("valid:", isValid);

const migratedJson = migrate_patch(patchJson);
console.log("migrated bytes:", migratedJson.length);
```

## 3) Read patch interface + compile summary

```ts
import init, { compile_patch, patch_interface } from "din-wasm";

await init();

const iface = patch_interface(patchJson) as {
  inputs: Array<{ key: string; node_id: string; param_id: string; default_value: number }>;
  outputs: Array<unknown>;
  events: Array<{ key: string }>;
};

console.log("inputs:", iface.inputs.map((i) => i.key));
console.log("events:", iface.events.map((e) => e.key));

const summary = compile_patch(patchJson) as {
  node_count: number;
  connection_count: number;
  audio_connection_count: number;
  transport_connection_count: number;
  trigger_connection_count: number;
  control_connection_count: number;
  transport_connected_ids: string[];
};

console.log(
  `${summary.node_count} nodes, ${summary.connection_count} connections, ${summary.audio_connection_count} audio edges`,
);
```

## 4) Render one audio block (stateless helper)

For simple/offline calls:

```ts
import init, { render_audio_block } from "din-wasm";

await init();

const block = render_audio_block(patchJson, 48_000, 2, 128);
console.log("samples:", block.length); // 256
```

## 5) Keep a runtime alive (stateful rendering)

Use `AudioRuntime` when you need to update controls between blocks.

```ts
import init, { AudioRuntime } from "din-wasm";

await init();

const runtime = AudioRuntime.fromPatch(patchJson, 48_000, 2, 128);

// If the patch interface contains these keys:
runtime.setInput("cutoff", 0.7);
runtime.triggerEvent("bang", 1n);
runtime.pushMidi(0x90, 60, 100, 0); // note on C4

const block = runtime.renderBlock();
console.log("non-zero:", block.some((v) => Math.abs(v) > 1e-4));

runtime.free();
```

## 6) Transport clock in TypeScript

```ts
import init, { TransportRuntime } from "din-wasm";

await init();

const transport = TransportRuntime.fromConfig(
  120, // bpm
  4,   // beats_per_bar
  4,   // beat_unit
  4,   // bars_per_phrase
  4,   // steps_per_beat
  0,   // swing
  "tick",
);

transport.play();
const dt = transport.secondsPerStep() * 2.2;
const ticks = transport.advanceSeconds(dt) as Array<{
  step_index: number;
  step_in_beat: number;
  beat_in_bar: number;
  bar_index: number;
  phrase_bar: number;
}>;

for (const tick of ticks) {
  console.log(`step=${tick.step_index} beat=${tick.beat_in_bar} bar=${tick.bar_index}`);
}

transport.free();
```

## 7) Utility helpers

```ts
import init, {
  audio_clamp,
  audio_compare,
  audio_math,
  audio_mix,
  audio_switch,
  midi_to_freq_value,
  midi_to_note_value,
  note_to_freq_value,
  note_to_midi_value,
  safe_identifier,
} from "din-wasm";

await init();

console.log(audio_math("multiply_add", 2, 3, 4)); // 10
console.log(audio_compare("greater_than", 4, 3)); // true
console.log(audio_mix(0, 10, 0.25, true)); // 2.5
console.log(audio_clamp(12, 0, 10, "clamp")); // 10
console.log(audio_switch(1, new Float32Array([4, 9, 16]))); // 9

console.log(note_to_midi_value("A4")); // 69
console.log(midi_to_note_value(61, true)); // Db4
console.log(midi_to_freq_value(69)); // 440
console.log(note_to_freq_value("La4")); // 440

console.log(safe_identifier("My Node!", "node")); // My_Node_
```

## 8) Error handling pattern

Wrap exported calls near boundaries:

```ts
function asErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

try {
  validate_patch("{not valid json");
} catch (error) {
  console.error("Patch validation failed:", asErrorMessage(error));
}
```

## 9) Notes on runtime v1 behavior

- Graphs containing a `patch` node are rejected by native runtime v1.
- Use `compile_patch`/`graph_from_patch` for static analysis flows.
- For production React usage, `react-din` remains the high-level TypeScript/React surface.

