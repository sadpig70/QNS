//! OpenQASM parser for QNS.
//!
//! This crate provides functionality to parse OpenQASM 2.0/3.0 code
//! and convert it into QNS `CircuitGenome`.

pub mod ast;
pub mod builder;
pub mod error;
pub mod parser;
pub mod preprocessor;

pub use builder::build_circuit;
pub use error::{QasmError, Result};
pub use parser::parse_qasm_str;
pub use preprocessor::resolve_includes;

use qns_core::CircuitGenome;

/// Parses OpenQASM source code and returns a CircuitGenome.
pub fn parse_qasm(source: &str) -> Result<CircuitGenome> {
    let ast = parse_qasm_str(source)?;
    build_circuit(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qns_core::types::Gate;

    #[test]
    fn test_parse_simple_circuit() {
        let source = r#"
            OPENQASM 2.0;
            qreg q[2];
            creg c[2];
            h q[0];
            cx q[0], q[1];
            measure q[0] -> c[0];
        "#;

        let circuit = parse_qasm(source).unwrap();

        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.gates.len(), 3);

        match &circuit.gates[0] {
            Gate::H(q) => assert_eq!(*q, 0),
            _ => panic!("Expected H gate"),
        }

        match &circuit.gates[1] {
            Gate::CNOT(c, t) => {
                assert_eq!(*c, 0);
                assert_eq!(*t, 1);
            },
            _ => panic!("Expected CNOT gate"),
        }

        match &circuit.gates[2] {
            Gate::Measure(q) => assert_eq!(*q, 0),
            _ => panic!("Expected Measure gate"),
        }
    }
}
