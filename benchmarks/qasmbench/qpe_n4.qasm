// Quantum Phase Estimation - 4 qubits
// Estimating eigenvalue of T gate
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[3];

// Initialize eigenstate |1⟩
x q[3];

// Hadamard on counting qubits
h q[0];
h q[1];
h q[2];

// Controlled-U operations (U = T gate)
// C-T^4
cp(pi/2) q[0],q[3];

// C-T^2
cp(pi/4) q[1],q[3];

// C-T
cp(pi/8) q[2],q[3];

// Inverse QFT on counting qubits
swap q[0],q[2];
h q[0];
cp(-pi/2) q[0],q[1];
h q[1];
cp(-pi/4) q[0],q[2];
cp(-pi/2) q[1],q[2];
h q[2];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];
