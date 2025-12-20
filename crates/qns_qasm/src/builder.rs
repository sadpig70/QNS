use crate::ast::*;
use crate::error::{QasmError, Result};
use qns_core::types::Gate;
use qns_core::CircuitGenome;
use std::collections::HashMap;

pub fn build_circuit(program: &Program) -> Result<CircuitGenome> {
    let builder = CircuitBuilder::new();
    builder.build(program)
}

struct CircuitBuilder {
    qubit_map: HashMap<String, (usize, usize)>, // name -> (start_index, size)
    total_qubits: usize,
    gates: Vec<Gate>,
}

impl CircuitBuilder {
    fn new() -> Self {
        Self {
            qubit_map: HashMap::new(),
            total_qubits: 0,
            gates: Vec::new(),
        }
    }

    fn build(mut self, program: &Program) -> Result<CircuitGenome> {
        // First pass: calculate total qubits and map registers
        for stmt in &program.statements {
            if let Statement::QRegDecl { name, size } = stmt {
                if self.qubit_map.contains_key(name) {
                    return Err(QasmError::BuildError(format!("Duplicate qreg '{}'", name)));
                }
                self.qubit_map
                    .insert(name.clone(), (self.total_qubits, *size));
                self.total_qubits += size;
            }
        }

        if self.total_qubits == 0 {
            // If no qregs, assume implicit qubits based on usage?
            // For now, require qreg.
            return Err(QasmError::BuildError(
                "No quantum registers declared".to_string(),
            ));
        }

        // Second pass: generate gates
        for stmt in &program.statements {
            match stmt {
                Statement::GateCall { name, params, args } => {
                    self.process_gate(name, params, args)?;
                },
                Statement::Measure { qubit, target: _ } => {
                    // For now, we only support measuring to cbits, but CircuitGenome
                    // handles measurement as a Gate::Measure(qubit).
                    // We ignore the target cbit for the simulation part in this MVP.
                    let q = self.resolve_qubit(qubit)?;
                    self.gates.push(Gate::Measure(q));
                },
                _ => {}, // Ignore other statements for MVP
            }
        }

        let mut circuit = CircuitGenome::new(self.total_qubits);
        for gate in self.gates {
            circuit.add_gate(gate).map_err(QasmError::QnsError)?;
        }

        Ok(circuit)
    }

    fn process_gate(&mut self, name: &str, params: &[f64], args: &[Argument]) -> Result<()> {
        let qubits: Vec<usize> = args
            .iter()
            .map(|arg| self.resolve_qubit(arg))
            .collect::<Result<_>>()?;

        match (name, qubits.len(), params.len()) {
            ("h", 1, 0) => self.gates.push(Gate::H(qubits[0])),
            ("x", 1, 0) => self.gates.push(Gate::X(qubits[0])),
            ("y", 1, 0) => self.gates.push(Gate::Y(qubits[0])),
            ("z", 1, 0) => self.gates.push(Gate::Z(qubits[0])),
            ("s", 1, 0) => self.gates.push(Gate::S(qubits[0])),
            ("t", 1, 0) => self.gates.push(Gate::T(qubits[0])),
            ("rx", 1, 1) => self.gates.push(Gate::Rx(qubits[0], params[0])),
            ("ry", 1, 1) => self.gates.push(Gate::Ry(qubits[0], params[0])),
            ("rz", 1, 1) => self.gates.push(Gate::Rz(qubits[0], params[0])),
            ("cx", 2, 0) => self.gates.push(Gate::CNOT(qubits[0], qubits[1])),
            ("cz", 2, 0) => self.gates.push(Gate::CZ(qubits[0], qubits[1])),
            ("swap", 2, 0) => self.gates.push(Gate::SWAP(qubits[0], qubits[1])),
            _ => {
                return Err(QasmError::BuildError(format!(
                    "Unknown or invalid gate: {} with {} args",
                    name,
                    qubits.len()
                )))
            },
        }
        Ok(())
    }

    fn resolve_qubit(&self, arg: &Argument) -> Result<usize> {
        match arg {
            Argument::Indexed(name, idx) => {
                let (start, size) = self
                    .qubit_map
                    .get(name)
                    .ok_or_else(|| QasmError::BuildError(format!("Undefined qreg '{}'", name)))?;
                if idx >= size {
                    return Err(QasmError::BuildError(format!(
                        "Index {} out of bounds for qreg '{}' of size {}",
                        idx, name, size
                    )));
                }
                Ok(start + idx)
            },
            Argument::Id(name) => {
                // If referenced without index, it usually implies broadcast or single qubit register
                // For MVP, we only support explicit indexing or size 1 registers
                let (start, size) = self
                    .qubit_map
                    .get(name)
                    .ok_or_else(|| QasmError::BuildError(format!("Undefined qreg '{}'", name)))?;
                if *size == 1 {
                    Ok(*start)
                } else {
                    Err(QasmError::BuildError(format!(
                        "Register '{}' used without index but has size {}",
                        name, size
                    )))
                }
            },
        }
    }
}
