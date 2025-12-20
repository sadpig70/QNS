// Quantum Supremacy Circuit - 4 qubits (simplified)
// Random circuit with depth 4
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[4];

// Layer 1
h q[0];
h q[1];
h q[2];
h q[3];

// Layer 2
cx q[0],q[1];
cx q[2],q[3];

// Layer 3
ry(0.7) q[0];
ry(1.2) q[1];
ry(0.5) q[2];
ry(0.9) q[3];

// Layer 4
cx q[1],q[2];
cx q[0],q[3];

// Layer 5
rz(0.3) q[0];
rz(0.8) q[1];
rz(1.1) q[2];
rz(0.4) q[3];

measure q -> c;
