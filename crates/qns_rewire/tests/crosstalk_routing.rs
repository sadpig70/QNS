use qns_core::prelude::*;
use qns_core::types::{CouplerProperties, CrosstalkMatrix, Fidelity, Topology};
use qns_rewire::router::{NoiseAwareRouter, Router};

#[test]
fn test_router_avoids_crosstalk() {
    // 4 Qubits:
    // 0 -- 1 (Strong Crosstalk with 2-3)
    // |    |
    // 2 -- 3
    //
    // Goal: CNOT(0, 1) and CNOT(2, 3) in parallel.
    // If we route CNOT(0, 1) directly, it might cause crosstalk if (0,1) interacts with (2,3).
    // Let's construct a topology where:
    // Path A: 0 -> 1 directly. (High Crosstalk with 2)
    // Path B: 0 -> 4 -> 1 (Hypothetical bypass? No, let's use edge scoring).

    // Scenario:
    // Qubit 0, 1, 2. Linear 0-1-2.
    // Gate 1: CNOT(0, 1)
    // Gate 2: X(2) happens at same time.
    // Crosstalk: (0, 1) <-> 2 is Strong.
    // Alternative: Maybe SWAP 0->something else?

    // Simpler Scenario for Routing path selection:
    // Source: 0, Target: 2.
    // Path A: 0 -> 1 -> 2. Edge (0,1) has high crosstalk with Qubit 3 (which is active).
    // Path B: 0 -> 4 -> 2. Edge (0,4) has NO crosstalk.
    // Edges (0,1) and (0,4) have same Fidelity (e.g. 99%).
    // Router should choose Path B.

    // Layout:
    // 1 -- 0 -- 4
    // |    |    |
    // 2    3    5
    //
    // We want 0 -> 2.
    // Path 1: 0 -> 1 -> 2. (Edge 0-1 is good, Edge 1-2 is good).
    // Path 2: 0 -> 3 -> ? (Dead end)
    // Path 3: 0 -> 4 -> 5 -> ? (Longer)

    // Let's try:
    //   1
    //  / \
    // 0   2
    //  \ /
    //   3
    // Target: 0 -> 2.
    // Path A: 0->1->2.
    // Path B: 0->3->2.
    //
    // Qubit 4 is active (running X gate).
    // Crosstalk: Edge (0,1) strongly interacts with Qubit 4.
    // Crosstalk: Edge (0,3) has 0 interaction with Qubit 4.

    let mut hw = HardwareProfile::new("test_ring", 5, Topology::Custom); // 0,1,2,3,4

    // Edges
    // 0-1, 1-2, 0-3, 3-2.
    // Helper to add coupler
    let mut add_edge = |u, v, fid| {
        let mut coupler = CouplerProperties::new(u, v);
        coupler.gate_fidelity = Fidelity::new(fid);
        hw.add_coupler(coupler);
    };

    add_edge(0, 1, 0.99);
    add_edge(1, 2, 0.99);
    add_edge(0, 3, 0.99);
    add_edge(3, 2, 0.99);

    // Qubit 4 is isolated but active

    // Crosstalk Matrix
    // Interaction (0, 1) -> 4 is HIGH
    let mut xtalk = CrosstalkMatrix::new();
    // When 0 is active, 4 suffers error (or vice versa).
    // We are routing CNOT(0, ?) maybe?
    // The router checks edges.
    // If we use edge (0,1), 0 is active.
    // If 4 Is active, (0,4) interaction makes (0,1) path costly.
    xtalk.set_interaction(0, 4, 0.5); // High penalty
    xtalk.set_interaction(1, 4, 0.5);

    // Path via 3 is clean
    // (0,3) -> 4 is 0.
    // (3,2) -> 4 is 0.

    hw.crosstalk = xtalk;

    // Circuit:
    // 1. CNOT(0, 2) (Requires routing 0->2)
    // 2. X(4) (Make 4 active - roughly parallel or in lookahead window)
    // We expect router to pick path 0->3->2 to avoid 0->1->2 (near 4).

    let mut circuit = CircuitGenome::new(5);
    circuit.add_gate(Gate::CNOT(0, 2)).unwrap();
    circuit.add_gate(Gate::X(4)).unwrap();

    // Router with high Crosstalk weight
    let router = NoiseAwareRouter::new(1.0, 0.0, 10.0); // High crosstalk penalty

    // To verify path, we need to inspect the routed circuit's used qubits/swaps.
    // Route
    let routed = router.route(&circuit, &hw).expect("Routing failed");

    // Analyze routed circuit
    // Gate 0: X(4) -> map[4]
    // Gate 1..N: SWAPs for CNOT(0,2)

    // Since X(4) is single qubit, it stays on logical 4 (identity map start).
    // If router chooses 0->3->2, it will likely insert SWAP(0,3) then CNOT(3,2) or similar.
    // If it chooses 0->1->2, it uses SWAP(0,1).

    println!("Routed gates: {:?}", routed.gates);

    // Check if Qubit 1 was used.
    // Logical 0 starts at Physical 0.
    // Logical 2 starts at Physical 2.
    // Logical 3 starts at Physical 3.
    // Logical 1 starts at Physical 1.

    // If path is 0->3->2, we expect interactions involving phy 3.
    // If path is 0->1->2, we expect interactions involving phy 1.

    let used_phy_1 = routed.gates.iter().any(|g| g.qubits().contains(&1));
    let used_phy_3 = routed.gates.iter().any(|g| g.qubits().contains(&3));

    assert!(
        !used_phy_1,
        "Router should avoid Physical 1 due to crosstalk"
    );
    assert!(used_phy_3, "Router should use Physical 3 (clean path)");
}
