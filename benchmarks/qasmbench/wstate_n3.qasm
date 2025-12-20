// W-State - 3 qubits
// From QASMBench Small category
OPENQASM 2.0;
include "qelib1.inc";

qreg q[3];
creg c[3];

// Create W-state: (|100⟩ + |010⟩ + |001⟩)/√3
ry(1.9106) q[0];
cx q[0],q[1];
x q[0];
ry(0.9553) q[0];
cx q[0],q[2];
cx q[1],q[2];
x q[1];

measure q -> c;
