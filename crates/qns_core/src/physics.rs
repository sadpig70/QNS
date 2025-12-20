//! Physical constants and gate matrices for quantum computing.
//!
//! This module provides:
//! - Standard gate matrices (Pauli, Clifford, rotation gates)
//! - Physical constants (typical T1/T2 values, gate times)
//! - Utility functions for matrix operations

use num_complex::Complex64;
use std::f64::consts::FRAC_1_SQRT_2;

/// Complex number type alias for readability.
pub type C64 = Complex64;

/// Create a complex number from real and imaginary parts.
#[inline]
pub const fn c(re: f64, im: f64) -> C64 {
    C64::new(re, im)
}

/// Complex zero.
pub const ZERO: C64 = C64::new(0.0, 0.0);

/// Complex one.
pub const ONE: C64 = C64::new(1.0, 0.0);

/// Complex imaginary unit i.
pub const I: C64 = C64::new(0.0, 1.0);

/// 1/√2 as complex number.
pub const FRAC_1_SQRT_2_C: C64 = C64::new(FRAC_1_SQRT_2, 0.0);

// ============================================================================
// Physical Constants
// ============================================================================

/// Typical T1 relaxation times for superconducting qubits (μs).
pub mod t1_typical {
    /// IBM Heron (2024): ~300 μs
    pub const IBM_HERON: f64 = 300.0;
    /// IBM Eagle (2022): ~100 μs
    pub const IBM_EAGLE: f64 = 100.0;
    /// Google Sycamore: ~20 μs
    pub const GOOGLE_SYCAMORE: f64 = 20.0;
    /// Rigetti Aspen: ~30 μs
    pub const RIGETTI_ASPEN: f64 = 30.0;
    /// Generic NISQ device
    pub const NISQ_TYPICAL: f64 = 100.0;
}

/// Typical T2 dephasing times for superconducting qubits (μs).
pub mod t2_typical {
    /// IBM Heron (2024): ~200 μs
    pub const IBM_HERON: f64 = 200.0;
    /// IBM Eagle (2022): ~80 μs
    pub const IBM_EAGLE: f64 = 80.0;
    /// Google Sycamore: ~10 μs
    pub const GOOGLE_SYCAMORE: f64 = 10.0;
    /// Rigetti Aspen: ~20 μs
    pub const RIGETTI_ASPEN: f64 = 20.0;
    /// Generic NISQ device
    pub const NISQ_TYPICAL: f64 = 80.0;
}

/// Typical gate times (ns).
pub mod gate_times {
    /// Single-qubit gate time
    pub const SINGLE_QUBIT: f64 = 35.0;
    /// Two-qubit gate time (CNOT, CZ)
    pub const TWO_QUBIT: f64 = 300.0;
    /// Measurement time
    pub const MEASUREMENT: f64 = 1000.0;
    /// Reset time
    pub const RESET: f64 = 1000.0;
}

/// Typical gate error rates.
pub mod gate_errors {
    /// Single-qubit gate error (state-of-the-art)
    pub const SINGLE_QUBIT_BEST: f64 = 0.0001;
    /// Single-qubit gate error (typical NISQ)
    pub const SINGLE_QUBIT_TYPICAL: f64 = 0.001;
    /// Two-qubit gate error (state-of-the-art)
    pub const TWO_QUBIT_BEST: f64 = 0.001;
    /// Two-qubit gate error (typical NISQ)
    pub const TWO_QUBIT_TYPICAL: f64 = 0.01;
    /// Measurement error (typical)
    pub const MEASUREMENT_TYPICAL: f64 = 0.01;
}

// ============================================================================
// Gate Matrices (2x2 for single-qubit, 4x4 for two-qubit)
// ============================================================================

/// 2x2 matrix type: [[a, b], [c, d]]
pub type Matrix2x2 = [[C64; 2]; 2];

/// 4x4 matrix type for two-qubit gates
pub type Matrix4x4 = [[C64; 4]; 4];

/// Identity matrix I.
pub const IDENTITY: Matrix2x2 = [[ONE, ZERO], [ZERO, ONE]];

/// Pauli-X (NOT) gate.
/// |0⟩ → |1⟩, |1⟩ → |0⟩
pub const PAULI_X: Matrix2x2 = [[ZERO, ONE], [ONE, ZERO]];

/// Pauli-Y gate.
/// |0⟩ → i|1⟩, |1⟩ → -i|0⟩
pub const PAULI_Y: Matrix2x2 = [[ZERO, C64::new(0.0, -1.0)], [I, ZERO]];

/// Pauli-Z gate.
/// |0⟩ → |0⟩, |1⟩ → -|1⟩
pub const PAULI_Z: Matrix2x2 = [[ONE, ZERO], [ZERO, C64::new(-1.0, 0.0)]];

/// Hadamard gate.
/// |0⟩ → (|0⟩ + |1⟩)/√2, |1⟩ → (|0⟩ - |1⟩)/√2
pub const HADAMARD: Matrix2x2 = [
    [FRAC_1_SQRT_2_C, FRAC_1_SQRT_2_C],
    [FRAC_1_SQRT_2_C, C64::new(-FRAC_1_SQRT_2, 0.0)],
];

/// S gate (√Z, phase gate).
/// |0⟩ → |0⟩, |1⟩ → i|1⟩
pub const S_GATE: Matrix2x2 = [[ONE, ZERO], [ZERO, I]];

/// S† (S-dagger) gate.
pub const S_DAGGER: Matrix2x2 = [[ONE, ZERO], [ZERO, C64::new(0.0, -1.0)]];

/// T gate (π/8 gate).
/// |0⟩ → |0⟩, |1⟩ → e^(iπ/4)|1⟩
pub const T_GATE: Matrix2x2 = [[ONE, ZERO], [ZERO, C64::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2)]];

/// T† (T-dagger) gate.
pub const T_DAGGER: Matrix2x2 = [[ONE, ZERO], [ZERO, C64::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2)]];

/// Computes Rx(θ) rotation matrix.
/// Rx(θ) = cos(θ/2)I - i·sin(θ/2)X
pub fn rx(theta: f64) -> Matrix2x2 {
    let cos = (theta / 2.0).cos();
    let sin = (theta / 2.0).sin();
    [[c(cos, 0.0), c(0.0, -sin)], [c(0.0, -sin), c(cos, 0.0)]]
}

/// Computes Ry(θ) rotation matrix.
/// Ry(θ) = cos(θ/2)I - i·sin(θ/2)Y
pub fn ry(theta: f64) -> Matrix2x2 {
    let cos = (theta / 2.0).cos();
    let sin = (theta / 2.0).sin();
    [[c(cos, 0.0), c(-sin, 0.0)], [c(sin, 0.0), c(cos, 0.0)]]
}

/// Computes Rz(θ) rotation matrix.
/// Rz(θ) = e^(-iθ/2)|0⟩⟨0| + e^(iθ/2)|1⟩⟨1|
pub fn rz(theta: f64) -> Matrix2x2 {
    let half = theta / 2.0;
    [
        [c(half.cos(), -half.sin()), ZERO],
        [ZERO, c(half.cos(), half.sin())],
    ]
}

/// CNOT (CX) gate matrix (control=0, target=1).
/// |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |11⟩, |11⟩ → |10⟩
pub const CNOT: Matrix4x4 = [
    [ONE, ZERO, ZERO, ZERO],
    [ZERO, ONE, ZERO, ZERO],
    [ZERO, ZERO, ZERO, ONE],
    [ZERO, ZERO, ONE, ZERO],
];

/// CZ gate matrix.
/// |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |10⟩, |11⟩ → -|11⟩
pub const CZ: Matrix4x4 = [
    [ONE, ZERO, ZERO, ZERO],
    [ZERO, ONE, ZERO, ZERO],
    [ZERO, ZERO, ONE, ZERO],
    [ZERO, ZERO, ZERO, C64::new(-1.0, 0.0)],
];

/// SWAP gate matrix.
/// |00⟩ → |00⟩, |01⟩ → |10⟩, |10⟩ → |01⟩, |11⟩ → |11⟩
pub const SWAP: Matrix4x4 = [
    [ONE, ZERO, ZERO, ZERO],
    [ZERO, ZERO, ONE, ZERO],
    [ZERO, ONE, ZERO, ZERO],
    [ZERO, ZERO, ZERO, ONE],
];

// ============================================================================
// Gate Classification for Commutativity
// ============================================================================

/// Gate classification for commutativity analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateType {
    /// Diagonal gates (Z, S, T, Rz) - commute with each other on same qubit
    Diagonal,
    /// X-axis rotation gates (X, Rx)
    XRotation,
    /// Y-axis rotation gates (Y, Ry)
    YRotation,
    /// Hadamard gate
    Hadamard,
    /// Two-qubit gates
    TwoQubit,
    /// Measurement
    Measurement,
}

/// Checks if two gate types commute when applied to the same qubit.
///
/// Note: This is a simplified check. Full commutativity depends on
/// the specific gate parameters (e.g., Rz(θ) commutes with Rz(φ) for any θ, φ).
pub fn gate_types_commute(t1: GateType, t2: GateType) -> bool {
    use GateType::*;
    matches!(
        (t1, t2),
        // Diagonal gates always commute with each other
        (Diagonal, Diagonal) |
        // X rotations commute with each other
        (XRotation, XRotation) |
        // Y rotations commute with each other
        (YRotation, YRotation)
    )
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Computes the 2x2 identity matrix.
pub fn identity_2x2() -> Matrix2x2 {
    IDENTITY
}

/// Computes the 4x4 identity matrix.
pub fn identity_4x4() -> Matrix4x4 {
    [
        [ONE, ZERO, ZERO, ZERO],
        [ZERO, ONE, ZERO, ZERO],
        [ZERO, ZERO, ONE, ZERO],
        [ZERO, ZERO, ZERO, ONE],
    ]
}

/// Computes the Hermitian conjugate (conjugate transpose) of a 2x2 matrix.
pub fn dagger_2x2(m: &Matrix2x2) -> Matrix2x2 {
    [
        [m[0][0].conj(), m[1][0].conj()],
        [m[0][1].conj(), m[1][1].conj()],
    ]
}

/// Multiplies two 2x2 matrices.
pub fn mul_2x2(a: &Matrix2x2, b: &Matrix2x2) -> Matrix2x2 {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

/// Checks if a 2x2 matrix is approximately unitary (U†U ≈ I).
pub fn is_unitary_2x2(m: &Matrix2x2, tolerance: f64) -> bool {
    let product = mul_2x2(&dagger_2x2(m), m);

    (product[0][0] - ONE).norm() < tolerance
        && product[0][1].norm() < tolerance
        && product[1][0].norm() < tolerance
        && (product[1][1] - ONE).norm() < tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const TOLERANCE: f64 = 1e-10;

    #[test]
    fn test_pauli_gates_unitary() {
        assert!(is_unitary_2x2(&PAULI_X, TOLERANCE));
        assert!(is_unitary_2x2(&PAULI_Y, TOLERANCE));
        assert!(is_unitary_2x2(&PAULI_Z, TOLERANCE));
    }

    #[test]
    fn test_hadamard_unitary() {
        assert!(is_unitary_2x2(&HADAMARD, TOLERANCE));
    }

    #[test]
    fn test_s_t_gates_unitary() {
        assert!(is_unitary_2x2(&S_GATE, TOLERANCE));
        assert!(is_unitary_2x2(&T_GATE, TOLERANCE));
    }

    #[test]
    fn test_rotation_gates_unitary() {
        for theta in [0.0, PI / 4.0, PI / 2.0, PI, 2.0 * PI] {
            assert!(
                is_unitary_2x2(&rx(theta), TOLERANCE),
                "Rx({}) not unitary",
                theta
            );
            assert!(
                is_unitary_2x2(&ry(theta), TOLERANCE),
                "Ry({}) not unitary",
                theta
            );
            assert!(
                is_unitary_2x2(&rz(theta), TOLERANCE),
                "Rz({}) not unitary",
                theta
            );
        }
    }

    #[test]
    fn test_x_squared_is_identity() {
        let x2 = mul_2x2(&PAULI_X, &PAULI_X);
        assert!((x2[0][0] - ONE).norm() < TOLERANCE);
        assert!(x2[0][1].norm() < TOLERANCE);
        assert!(x2[1][0].norm() < TOLERANCE);
        assert!((x2[1][1] - ONE).norm() < TOLERANCE);
    }

    #[test]
    fn test_z_squared_is_identity() {
        let z2 = mul_2x2(&PAULI_Z, &PAULI_Z);
        assert!((z2[0][0] - ONE).norm() < TOLERANCE);
        assert!((z2[1][1] - ONE).norm() < TOLERANCE);
    }

    #[test]
    fn test_s_squared_is_z() {
        let s2 = mul_2x2(&S_GATE, &S_GATE);
        assert!((s2[0][0] - PAULI_Z[0][0]).norm() < TOLERANCE);
        assert!((s2[1][1] - PAULI_Z[1][1]).norm() < TOLERANCE);
    }

    #[test]
    fn test_hadamard_squared_is_identity() {
        let h2 = mul_2x2(&HADAMARD, &HADAMARD);
        assert!((h2[0][0] - ONE).norm() < TOLERANCE);
        assert!(h2[0][1].norm() < TOLERANCE);
        assert!(h2[1][0].norm() < TOLERANCE);
        assert!((h2[1][1] - ONE).norm() < TOLERANCE);
    }

    #[test]
    fn test_diagonal_gates_commute() {
        assert!(gate_types_commute(GateType::Diagonal, GateType::Diagonal));
    }
}
