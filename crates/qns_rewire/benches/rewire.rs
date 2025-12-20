//! Rewire benchmark.
//!
//! Target: <20ms for gate reordering

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qns_core::prelude::*;
use qns_rewire::{GateReorder, LiveRewirer, ReorderConfig};

fn create_circuit(num_gates: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(5);

    for i in 0..num_gates {
        match i % 5 {
            0 => circuit.add_gate(Gate::H(i % 5)).unwrap(),
            1 => circuit.add_gate(Gate::X((i + 1) % 5)).unwrap(),
            2 => circuit.add_gate(Gate::Z((i + 2) % 5)).unwrap(),
            3 => circuit.add_gate(Gate::CNOT(i % 5, (i + 1) % 5)).unwrap(),
            _ => circuit.add_gate(Gate::T((i + 3) % 5)).unwrap(),
        }
    }

    circuit
}

fn bench_find_commuting_pairs(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_commuting_pairs");

    for num_gates in [10, 20, 50, 100] {
        let circuit = create_circuit(num_gates);
        let reorder = GateReorder::default();

        group.bench_with_input(
            BenchmarkId::new("gates", num_gates),
            &circuit,
            |b, circuit| b.iter(|| reorder.find_adjacent_commuting_pairs(circuit)),
        );
    }

    group.finish();
}

fn bench_generate_reorderings(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_reorderings");

    for num_gates in [10, 20, 50] {
        let circuit = create_circuit(num_gates);
        let reorder = GateReorder::with_config(ReorderConfig {
            max_variants: 50,
            max_depth: 3,
            deduplicate: true,
        });

        group.bench_with_input(
            BenchmarkId::new("gates", num_gates),
            &circuit,
            |b, circuit| b.iter(|| reorder.generate_reorderings(circuit)),
        );
    }

    group.finish();
}

fn bench_live_rewirer_optimize(c: &mut Criterion) {
    let circuit = create_circuit(20);
    let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

    c.bench_function("live_rewirer_optimize_20", |b| {
        b.iter(|| {
            let mut rewirer = LiveRewirer::new();
            rewirer.load(circuit.clone()).unwrap();
            rewirer.optimize(&noise, 20).unwrap()
        })
    });
}

fn bench_live_rewirer_with_hardware(c: &mut Criterion) {
    let circuit = create_circuit(20);
    let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
    let hw = HardwareProfile::linear("test", 5);

    c.bench_function("live_rewirer_hardware_aware", |b| {
        b.iter(|| {
            let mut rewirer = LiveRewirer::new();
            rewirer.set_hardware(hw.clone());
            rewirer.load(circuit.clone()).unwrap();
            rewirer.optimize(&noise, 20).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_find_commuting_pairs,
    bench_generate_reorderings,
    bench_live_rewirer_optimize,
    bench_live_rewirer_with_hardware,
);
criterion_main!(benches);
