// Deutsch-Jozsa Algorithm - 4 qubits (3 data + 1 ancilla)
// Testing if function is constant or balanced
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[3];

// Initialize
x q[3];
h q[0];
h q[1];
h q[2];
h q[3];

// Oracle for balanced function
cx q[0],q[3];
cx q[1],q[3];

// Hadamard on data qubits
h q[0];
h q[1];
h q[2];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];
