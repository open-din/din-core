//! Audio block render throughput (native engine). Run: `cargo bench -p din-core --bench engine_render`.
#![allow(missing_docs)]

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use din_core::{CompiledGraph, Engine, EngineConfig, PatchImporter};

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

fn bench_render_block(c: &mut Criterion) {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture");
    let compiled = CompiledGraph::from_patch(&patch).expect("compile");
    let config = EngineConfig {
        sample_rate: 48_000.0,
        channels: 2,
        block_size: 128,
    };
    let mut engine = Engine::new(compiled, config).expect("engine");
    let expected_len = engine.interleaved_output_len();
    let mut dst = vec![0.0f32; expected_len];

    let mut group = c.benchmark_group("engine_render_block");
    group.throughput(Throughput::Elements(dst.len() as u64));
    group.bench_function("render_block_into_reused_buffer", |b| {
        b.iter(|| {
            engine
                .render_block_into(black_box(&mut dst))
                .expect("render");
            black_box(dst[0])
        });
    });
    group.finish();
}

criterion_group!(benches, bench_render_block);
criterion_main!(benches);
