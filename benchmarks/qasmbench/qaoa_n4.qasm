// QAOA (Quantum Approximate Optimization Algorithm) - 4 qubits
// MaxCut problem on a simple graph
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[4];

// Initial state: equal superposition
h q[0];
h q[1];
h q[2];
h q[3];

// Problem Hamiltonian (gamma = 0.5)
// Edges: (0,1), (1,2), (2,3), (3,0)
cx q[0],q[1];
rz(0.5) q[1];
cx q[0],q[1];

cx q[1],q[2];
rz(0.5) q[2];
cx q[1],q[2];

cx q[2],q[3];
rz(0.5) q[3];
cx q[2],q[3];

cx q[3],q[0];
rz(0.5) q[0];
cx q[3],q[0];

// Mixer Hamiltonian (beta = 0.3)
rx(0.6) q[0];
rx(0.6) q[1];
rx(0.6) q[2];
rx(0.6) q[3];

measure q -> c;
