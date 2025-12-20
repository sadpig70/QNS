OPENQASM 2.0;
include "qelib1.inc";

// 4-qubit Quantum Fourier Transform
// Tests: Rotation gates, complex gate sequences, phase relationships

qreg q[4];
creg c[4];

// QFT implementation for 4 qubits
// Stage 1: q[0]
h q[0];
cu1(pi/2) q[1], q[0];
cu1(pi/4) q[2], q[0];
cu1(pi/8) q[3], q[0];

// Stage 2: q[1]
h q[1];
cu1(pi/2) q[2], q[1];
cu1(pi/4) q[3], q[1];

// Stage 3: q[2]
h q[2];
cu1(pi/2) q[3], q[2];

// Stage 4: q[3]
h q[3];

// SWAP for bit reversal
swap q[0], q[3];
swap q[1], q[2];

// Measure
measure q -> c;
