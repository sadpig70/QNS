//! Quantum gate definitions.
//!
//! This module provides quantum gate types with:
//! - Matrix representations for simulation
//! - Commutativity analysis for circuit optimization
//! - Inverse gate computation

use crate::physics::{self, GateType, Matrix2x2, Matrix4x4};
use serde::{Deserialize, Serialize};

/// Quantum gate enumeration.
///
/// Supports 12 gate types:
/// - Single-qubit: H, X, Y, Z, S, T, Rx, Ry, Rz
/// - Two-qubit: CNOT, CZ, SWAP
/// - Measurement: Measure
///
/// # Example
///
/// ```
/// use qns_core::prelude::*;
///
/// let h = Gate::H(0);
/// let cnot = Gate::CNOT(0, 1);
///
/// // Check commutativity
/// assert!(h.commutes_with(&Gate::X(1)));  // Different qubits
/// assert!(!h.commutes_with(&Gate::X(0))); // Same qubit, different axis
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gate {
    // Single-qubit gates (Clifford)
    /// Hadamard gate
    H(usize),
    /// Pauli-X (NOT) gate
    X(usize),
    /// Pauli-Y gate
    Y(usize),
    /// Pauli-Z gate
    Z(usize),
    /// S gate (√Z, phase gate)
    S(usize),
    /// T gate (π/8 gate)
    T(usize),

    // Single-qubit rotation gates
    /// Rotation around X-axis by angle θ
    Rx(usize, f64),
    /// Rotation around Y-axis by angle θ
    Ry(usize, f64),
    /// Rotation around Z-axis by angle θ
    Rz(usize, f64),

    // Two-qubit gates
    /// Controlled-NOT gate (control, target)
    CNOT(usize, usize),
    /// Controlled-Z gate
    CZ(usize, usize),
    /// SWAP gate
    SWAP(usize, usize),

    // Measurement
    /// Measurement in computational basis
    Measure(usize),
}

impl Gate {
    /// Returns the qubit indices this gate operates on.
    ///
    /// For single-qubit gates, returns a single-element vector.
    /// For two-qubit gates, returns [control, target] or [qubit1, qubit2].
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Gate::H(q)
            | Gate::X(q)
            | Gate::Y(q)
            | Gate::Z(q)
            | Gate::S(q)
            | Gate::T(q)
            | Gate::Rx(q, _)
            | Gate::Ry(q, _)
            | Gate::Rz(q, _)
            | Gate::Measure(q) => vec![*q],
            Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => vec![*c, *t],
        }
    }

    /// Returns the gate type for commutativity analysis.
    pub fn gate_type(&self) -> GateType {
        match self {
            Gate::Z(_) | Gate::S(_) | Gate::T(_) | Gate::Rz(_, _) => GateType::Diagonal,
            Gate::X(_) | Gate::Rx(_, _) => GateType::XRotation,
            Gate::Y(_) | Gate::Ry(_, _) => GateType::YRotation,
            Gate::H(_) => GateType::Hadamard,
            Gate::CNOT(_, _) | Gate::CZ(_, _) | Gate::SWAP(_, _) => GateType::TwoQubit,
            Gate::Measure(_) => GateType::Measurement,
        }
    }

    /// Checks if this gate commutes with another gate.
    ///
    /// Two gates commute if applying them in either order produces
    /// the same result. This check considers:
    /// 1. Gates on disjoint qubits always commute
    /// 2. Diagonal gates (Z, S, T, Rz) commute with each other
    /// 3. Same-axis rotations commute (Rx with Rx, etc.)
    ///
    /// # Example
    ///
    /// ```
    /// use qns_core::prelude::*;
    ///
    /// // Different qubits - always commute
    /// assert!(Gate::H(0).commutes_with(&Gate::X(1)));
    ///
    /// // Diagonal gates on same qubit - commute
    /// assert!(Gate::Z(0).commutes_with(&Gate::Rz(0, 0.5)));
    /// assert!(Gate::S(0).commutes_with(&Gate::T(0)));
    ///
    /// // Different axes on same qubit - don't commute
    /// assert!(!Gate::X(0).commutes_with(&Gate::Z(0)));
    /// ```
    pub fn commutes_with(&self, other: &Gate) -> bool {
        let q1 = self.qubits();
        let q2 = other.qubits();

        // Gates on disjoint qubits always commute
        if q1.iter().all(|q| !q2.contains(q)) {
            return true;
        }

        // Measurement doesn't commute with anything on same qubit
        if matches!(self, Gate::Measure(_)) || matches!(other, Gate::Measure(_)) {
            return false;
        }

        // Check gate type compatibility for same-qubit operations
        let t1 = self.gate_type();
        let t2 = other.gate_type();

        physics::gate_types_commute(t1, t2)
    }

    /// Returns true if this is a single-qubit gate.
    pub fn is_single_qubit(&self) -> bool {
        matches!(
            self,
            Gate::H(_)
                | Gate::X(_)
                | Gate::Y(_)
                | Gate::Z(_)
                | Gate::S(_)
                | Gate::T(_)
                | Gate::Rx(_, _)
                | Gate::Ry(_, _)
                | Gate::Rz(_, _)
        )
    }

    /// Returns true if this is a two-qubit gate.
    pub fn is_two_qubit(&self) -> bool {
        matches!(self, Gate::CNOT(_, _) | Gate::CZ(_, _) | Gate::SWAP(_, _))
    }

    /// Returns true if this is a measurement operation.
    pub fn is_measurement(&self) -> bool {
        matches!(self, Gate::Measure(_))
    }

    /// Returns true if this is a Clifford gate.
    ///
    /// Clifford gates are: H, S, CNOT, and their compositions.
    /// X, Y, Z are also Clifford (Paulis).
    pub fn is_clifford(&self) -> bool {
        matches!(
            self,
            Gate::H(_)
                | Gate::X(_)
                | Gate::Y(_)
                | Gate::Z(_)
                | Gate::S(_)
                | Gate::CNOT(_, _)
                | Gate::CZ(_, _)
                | Gate::SWAP(_, _)
        )
    }

    /// Returns the 2x2 matrix representation for single-qubit gates.
    ///
    /// Returns `None` for two-qubit gates and measurements.
    pub fn matrix_2x2(&self) -> Option<Matrix2x2> {
        match self {
            Gate::H(_) => Some(physics::HADAMARD),
            Gate::X(_) => Some(physics::PAULI_X),
            Gate::Y(_) => Some(physics::PAULI_Y),
            Gate::Z(_) => Some(physics::PAULI_Z),
            Gate::S(_) => Some(physics::S_GATE),
            Gate::T(_) => Some(physics::T_GATE),
            Gate::Rx(_, theta) => Some(physics::rx(*theta)),
            Gate::Ry(_, theta) => Some(physics::ry(*theta)),
            Gate::Rz(_, theta) => Some(physics::rz(*theta)),
            _ => None,
        }
    }

    /// Returns the 4x4 matrix representation for two-qubit gates.
    ///
    /// Returns `None` for single-qubit gates and measurements.
    /// The matrix assumes standard ordering (control < target for CNOT).
    pub fn matrix_4x4(&self) -> Option<Matrix4x4> {
        match self {
            Gate::CNOT(_, _) => Some(physics::CNOT),
            Gate::CZ(_, _) => Some(physics::CZ),
            Gate::SWAP(_, _) => Some(physics::SWAP),
            _ => None,
        }
    }

    /// Returns the inverse (adjoint) of this gate.
    ///
    /// For unitary gates, the inverse satisfies U†U = I.
    /// Returns `None` for measurements (not reversible).
    pub fn inverse(&self) -> Option<Gate> {
        match self {
            // Self-inverse gates
            Gate::H(q) => Some(Gate::H(*q)),
            Gate::X(q) => Some(Gate::X(*q)),
            Gate::Y(q) => Some(Gate::Y(*q)),
            Gate::Z(q) => Some(Gate::Z(*q)),
            Gate::CNOT(c, t) => Some(Gate::CNOT(*c, *t)),
            Gate::CZ(c, t) => Some(Gate::CZ(*c, *t)),
            Gate::SWAP(a, b) => Some(Gate::SWAP(*a, *b)),

            // S† and T†
            Gate::S(q) => Some(Gate::Rz(*q, -std::f64::consts::FRAC_PI_2)),
            Gate::T(q) => Some(Gate::Rz(*q, -std::f64::consts::FRAC_PI_4)),

            // Rotation inverses: negate the angle
            Gate::Rx(q, theta) => Some(Gate::Rx(*q, -theta)),
            Gate::Ry(q, theta) => Some(Gate::Ry(*q, -theta)),
            Gate::Rz(q, theta) => Some(Gate::Rz(*q, -theta)),

            // Measurement is not reversible
            Gate::Measure(_) => None,
        }
    }

    /// Returns the rotation angle for rotation gates.
    ///
    /// Returns `None` for non-rotation gates.
    pub fn rotation_angle(&self) -> Option<f64> {
        match self {
            Gate::Rx(_, theta) | Gate::Ry(_, theta) | Gate::Rz(_, theta) => Some(*theta),
            Gate::S(_) => Some(std::f64::consts::FRAC_PI_2),
            Gate::T(_) => Some(std::f64::consts::FRAC_PI_4),
            Gate::Z(_) => Some(std::f64::consts::PI),
            _ => None,
        }
    }

    /// Estimates the gate time in nanoseconds.
    pub fn estimated_time_ns(&self) -> f64 {
        if self.is_single_qubit() {
            physics::gate_times::SINGLE_QUBIT
        } else if self.is_two_qubit() {
            physics::gate_times::TWO_QUBIT
        } else {
            physics::gate_times::MEASUREMENT
        }
    }

    /// Estimates the gate error rate.
    pub fn estimated_error(&self) -> f64 {
        if self.is_single_qubit() {
            physics::gate_errors::SINGLE_QUBIT_TYPICAL
        } else if self.is_two_qubit() {
            physics::gate_errors::TWO_QUBIT_TYPICAL
        } else {
            physics::gate_errors::MEASUREMENT_TYPICAL
        }
    }

    /// Remaps the qubits of the gate using the provided mapping.
    /// mapping[old_qubit_index] = new_qubit_index
    pub fn map_qubits(&self, mapping: &[usize]) -> Gate {
        match self {
            Gate::H(q) => Gate::H(mapping[*q]),
            Gate::X(q) => Gate::X(mapping[*q]),
            Gate::Y(q) => Gate::Y(mapping[*q]),
            Gate::Z(q) => Gate::Z(mapping[*q]),
            Gate::S(q) => Gate::S(mapping[*q]),
            Gate::T(q) => Gate::T(mapping[*q]),
            Gate::Rx(q, theta) => Gate::Rx(mapping[*q], *theta),
            Gate::Ry(q, theta) => Gate::Ry(mapping[*q], *theta),
            Gate::Rz(q, theta) => Gate::Rz(mapping[*q], *theta),
            Gate::CNOT(c, t) => Gate::CNOT(mapping[*c], mapping[*t]),
            Gate::CZ(c, t) => Gate::CZ(mapping[*c], mapping[*t]),
            Gate::SWAP(a, b) => Gate::SWAP(mapping[*a], mapping[*b]),
            Gate::Measure(q) => Gate::Measure(mapping[*q]),
        }
    }
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::H(q) => write!(f, "H({})", q),
            Gate::X(q) => write!(f, "X({})", q),
            Gate::Y(q) => write!(f, "Y({})", q),
            Gate::Z(q) => write!(f, "Z({})", q),
            Gate::S(q) => write!(f, "S({})", q),
            Gate::T(q) => write!(f, "T({})", q),
            Gate::Rx(q, theta) => write!(f, "Rx({}, {:.4})", q, theta),
            Gate::Ry(q, theta) => write!(f, "Ry({}, {:.4})", q, theta),
            Gate::Rz(q, theta) => write!(f, "Rz({}, {:.4})", q, theta),
            Gate::CNOT(c, t) => write!(f, "CNOT({}, {})", c, t),
            Gate::CZ(c, t) => write!(f, "CZ({}, {})", c, t),
            Gate::SWAP(a, b) => write!(f, "SWAP({}, {})", a, b),
            Gate::Measure(q) => write!(f, "Measure({})", q),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_gate_qubits() {
        assert_eq!(Gate::H(0).qubits(), vec![0]);
        assert_eq!(Gate::CNOT(0, 1).qubits(), vec![0, 1]);
        assert_eq!(Gate::SWAP(2, 5).qubits(), vec![2, 5]);
    }

    #[test]
    fn test_commutes_different_qubits() {
        // Gates on different qubits always commute
        assert!(Gate::H(0).commutes_with(&Gate::X(1)));
        assert!(Gate::CNOT(0, 1).commutes_with(&Gate::H(2)));
        assert!(Gate::Rz(0, 0.5).commutes_with(&Gate::Ry(1, 0.3)));
    }

    #[test]
    fn test_commutes_diagonal_gates() {
        // Diagonal gates (Z, S, T, Rz) commute on same qubit
        assert!(Gate::Z(0).commutes_with(&Gate::S(0)));
        assert!(Gate::S(0).commutes_with(&Gate::T(0)));
        assert!(Gate::T(0).commutes_with(&Gate::Rz(0, 0.5)));
        assert!(Gate::Rz(0, 0.1).commutes_with(&Gate::Rz(0, 0.2)));
    }

    #[test]
    fn test_commutes_same_axis() {
        // Same-axis rotations commute
        assert!(Gate::X(0).commutes_with(&Gate::Rx(0, 0.5)));
        assert!(Gate::Rx(0, 0.1).commutes_with(&Gate::Rx(0, 0.2)));
        assert!(Gate::Y(0).commutes_with(&Gate::Ry(0, 0.5)));
    }

    #[test]
    fn test_not_commutes_different_axes() {
        // Different axes don't commute
        assert!(!Gate::X(0).commutes_with(&Gate::Y(0)));
        assert!(!Gate::X(0).commutes_with(&Gate::Z(0)));
        assert!(!Gate::H(0).commutes_with(&Gate::X(0)));
    }

    #[test]
    fn test_measurement_never_commutes() {
        assert!(!Gate::Measure(0).commutes_with(&Gate::H(0)));
        assert!(!Gate::Measure(0).commutes_with(&Gate::Z(0)));
    }

    #[test]
    fn test_matrix_single_qubit() {
        assert!(Gate::H(0).matrix_2x2().is_some());
        assert!(Gate::X(0).matrix_2x2().is_some());
        assert!(Gate::Rz(0, 0.5).matrix_2x2().is_some());
        assert!(Gate::CNOT(0, 1).matrix_2x2().is_none());
    }

    #[test]
    fn test_matrix_two_qubit() {
        assert!(Gate::CNOT(0, 1).matrix_4x4().is_some());
        assert!(Gate::CZ(0, 1).matrix_4x4().is_some());
        assert!(Gate::SWAP(0, 1).matrix_4x4().is_some());
        assert!(Gate::H(0).matrix_4x4().is_none());
    }

    #[test]
    fn test_inverse() {
        // Self-inverse gates
        assert_eq!(Gate::H(0).inverse(), Some(Gate::H(0)));
        assert_eq!(Gate::X(0).inverse(), Some(Gate::X(0)));
        assert_eq!(Gate::CNOT(0, 1).inverse(), Some(Gate::CNOT(0, 1)));

        // Rotation inverses
        let rx = Gate::Rx(0, 0.5);
        if let Some(Gate::Rx(q, theta)) = rx.inverse() {
            assert_eq!(q, 0);
            assert!((theta + 0.5).abs() < 1e-10);
        } else {
            panic!("Expected Rx inverse");
        }

        // Measurement has no inverse
        assert_eq!(Gate::Measure(0).inverse(), None);
    }

    #[test]
    fn test_is_clifford() {
        assert!(Gate::H(0).is_clifford());
        assert!(Gate::S(0).is_clifford());
        assert!(Gate::CNOT(0, 1).is_clifford());
        assert!(!Gate::T(0).is_clifford());
        assert!(!Gate::Rx(0, 0.5).is_clifford());
    }

    #[test]
    fn test_rotation_angle() {
        assert_eq!(Gate::Rx(0, 0.5).rotation_angle(), Some(0.5));
        assert_eq!(Gate::Rz(0, PI).rotation_angle(), Some(PI));
        assert!(Gate::S(0).rotation_angle().is_some());
        assert_eq!(Gate::H(0).rotation_angle(), None);
    }
}
