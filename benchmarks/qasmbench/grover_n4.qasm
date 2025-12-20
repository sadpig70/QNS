// Grover's Algorithm - 4 qubits (3 data + 1 ancilla)
// Searching for |11⟩ in 2-qubit space
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[2];

// Initialize: superposition on data qubits, |1⟩ on ancilla
h q[0];
h q[1];
x q[2];
h q[2];

// Oracle: mark |11⟩
x q[0];
x q[1];
ccx q[0],q[1],q[2];
x q[0];
x q[1];

// Diffusion operator
h q[0];
h q[1];
x q[0];
x q[1];
cz q[0],q[1];
x q[0];
x q[1];
h q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
