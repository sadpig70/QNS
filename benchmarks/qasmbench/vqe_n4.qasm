// Variational Quantum Eigensolver (VQE) - 4 qubits
// From QASMBench Small category
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[4];

// Ansatz with parameterized gates
ry(0.5) q[0];
ry(0.5) q[1];
ry(0.5) q[2];
ry(0.5) q[3];

cx q[0],q[1];
cx q[1],q[2];
cx q[2],q[3];

ry(1.0) q[0];
ry(1.0) q[1];
ry(1.0) q[2];
ry(1.0) q[3];

cx q[0],q[1];
cx q[1],q[2];
cx q[2],q[3];

measure q -> c;
