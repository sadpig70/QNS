//! Simulator benchmark.
//!
//! Target: <50ms for 5-qubit circuits

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qns_core::prelude::*;
use qns_simulator::{NoiseModel, NoisySimulator, StateVectorSimulator};

fn create_random_circuit(num_qubits: usize, num_gates: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(num_qubits);

    for i in 0..num_gates {
        match i % 6 {
            0 => circuit.add_gate(Gate::H(i % num_qubits)).unwrap(),
            1 => circuit.add_gate(Gate::X(i % num_qubits)).unwrap(),
            2 => circuit.add_gate(Gate::T(i % num_qubits)).unwrap(),
            3 => circuit
                .add_gate(Gate::Rz((i + 1) % num_qubits, 0.5))
                .unwrap(),
            4 if num_qubits > 1 => circuit
                .add_gate(Gate::CNOT(i % num_qubits, (i + 1) % num_qubits))
                .unwrap(),
            _ => circuit.add_gate(Gate::S(i % num_qubits)).unwrap(),
        }
    }

    circuit
}

fn bench_single_qubit_gates(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_qubit_gates");

    for num_qubits in [3, 5, 8, 10] {
        let mut sim = StateVectorSimulator::new(num_qubits);

        group.bench_with_input(
            BenchmarkId::new("H_gate", num_qubits),
            &num_qubits,
            |b, _| {
                b.iter(|| {
                    sim.reset();
                    sim.apply_gate(&Gate::H(0)).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_two_qubit_gates(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_qubit_gates");

    for num_qubits in [3, 5, 8, 10] {
        let mut sim = StateVectorSimulator::new(num_qubits);

        group.bench_with_input(BenchmarkId::new("CNOT", num_qubits), &num_qubits, |b, _| {
            b.iter(|| {
                sim.reset();
                sim.apply_gate(&Gate::CNOT(0, 1)).unwrap()
            })
        });
    }

    group.finish();
}

fn bench_execute_circuit(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute_circuit");

    // Test with different qubit counts
    for num_qubits in [3, 5, 8] {
        let circuit = create_random_circuit(num_qubits, 20);

        group.bench_with_input(
            BenchmarkId::new("qubits", num_qubits),
            &circuit,
            |b, circuit| {
                b.iter(|| {
                    let mut sim = StateVectorSimulator::new(circuit.num_qubits);
                    sim.execute(circuit).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("measurement");

    for num_qubits in [3, 5, 8] {
        let mut sim = StateVectorSimulator::new(num_qubits);
        sim.prepare_ghz_state().unwrap();

        group.bench_with_input(
            BenchmarkId::new("measure_1000_shots", num_qubits),
            &num_qubits,
            |b, _| b.iter(|| sim.measure(1000).unwrap()),
        );
    }

    group.finish();
}

fn bench_bell_state(c: &mut Criterion) {
    c.bench_function("prepare_bell_state", |b| {
        b.iter(|| {
            let mut sim = StateVectorSimulator::new(2);
            sim.prepare_bell_state().unwrap()
        })
    });
}

fn bench_ghz_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("ghz_state");

    for num_qubits in [3, 5, 8, 10] {
        group.bench_with_input(
            BenchmarkId::new("qubits", num_qubits),
            &num_qubits,
            |b, &n| {
                b.iter(|| {
                    let mut sim = StateVectorSimulator::new(n);
                    sim.prepare_ghz_state().unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_fidelity(c: &mut Criterion) {
    let mut group = c.benchmark_group("fidelity");

    for num_qubits in [3, 5, 8] {
        let sim1 = StateVectorSimulator::new(num_qubits);
        let sim2 = StateVectorSimulator::new(num_qubits);

        group.bench_with_input(
            BenchmarkId::new("qubits", num_qubits),
            &num_qubits,
            |b, _| b.iter(|| sim1.fidelity_with(&sim2).unwrap()),
        );
    }

    group.finish();
}

fn bench_full_workflow(c: &mut Criterion) {
    // Benchmark complete workflow: create, execute, measure
    let circuit = create_random_circuit(5, 30);

    c.bench_function("full_workflow_5qubits", |b| {
        b.iter(|| {
            let mut sim = StateVectorSimulator::new(5);
            sim.execute(&circuit).unwrap();
            sim.measure(100).unwrap()
        })
    });
}

// === Noisy Simulator Benchmarks ===

fn bench_noisy_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("noisy_execute");
    let noise = NoiseModel::new();

    for num_qubits in [3, 5] {
        let circuit = create_random_circuit(num_qubits, 20);

        group.bench_with_input(
            BenchmarkId::new("qubits", num_qubits),
            &circuit,
            |b, circuit| {
                b.iter(|| {
                    let mut sim = NoisySimulator::new(circuit.num_qubits, noise.clone());
                    sim.execute(circuit).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_noisy_vs_ideal(c: &mut Criterion) {
    let circuit = create_random_circuit(5, 20);
    let noise = NoiseModel::with_t1t2(100.0, 80.0);

    let mut group = c.benchmark_group("noisy_vs_ideal");

    group.bench_function("ideal_5q", |b| {
        b.iter(|| {
            let mut sim = StateVectorSimulator::new(5);
            sim.execute(&circuit).unwrap()
        })
    });

    group.bench_function("noisy_5q", |b| {
        b.iter(|| {
            let mut sim = NoisySimulator::new(5, noise.clone());
            sim.execute(&circuit).unwrap()
        })
    });

    group.finish();
}

fn bench_noisy_measurement(c: &mut Criterion) {
    let noise = NoiseModel::new().with_readout_error(0.01);
    let mut sim = NoisySimulator::new(5, noise);

    // Prepare a state
    sim.apply_gate(&Gate::H(0)).unwrap();
    sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();
    sim.apply_gate(&Gate::CNOT(1, 2)).unwrap();

    c.bench_function("noisy_measure_1000_shots", |b| {
        b.iter(|| sim.measure(1000).unwrap())
    });
}

criterion_group!(
    benches,
    bench_single_qubit_gates,
    bench_two_qubit_gates,
    bench_execute_circuit,
    bench_measurement,
    bench_bell_state,
    bench_ghz_state,
    bench_fidelity,
    bench_full_workflow,
    bench_noisy_execute,
    bench_noisy_vs_ideal,
    bench_noisy_measurement,
);
criterion_main!(benches);
