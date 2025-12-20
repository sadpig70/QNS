use crate::NoiseChannel;
use qns_core::types::Gate;

/// Bit Flip Channel: Flips the qubit (X gate) with probability p.
pub struct BitFlip {
    pub p: f64,
}

impl NoiseChannel for BitFlip {
    fn apply(&self, _gate: &Gate) -> Vec<(f64, Gate)> {
        vec![
            // 1. No Error (Identity) - using Rz(0, 0.0) as placeholder
            (1.0 - self.p, Gate::Rz(0, 0.0)),
            // 2. Bit Flip (X)
            (self.p, Gate::X(0)),
        ]
    }

    fn name(&self) -> &str {
        "BitFlip"
    }
}

/// Phase Flip Channel: Flips the phase (Z gate) with probability p.
pub struct PhaseFlip {
    pub p: f64,
}

impl NoiseChannel for PhaseFlip {
    fn apply(&self, _gate: &Gate) -> Vec<(f64, Gate)> {
        vec![
            // 1. No Error
            (1.0 - self.p, Gate::Rz(0, 0.0)),
            // 2. Phase Flip (Z)
            (self.p, Gate::Z(0)),
        ]
    }

    fn name(&self) -> &str {
        "PhaseFlip"
    }
}

/// Depolarizing Channel: Applies X, Y, or Z with probability p/3 each.
pub struct Depolarizing {
    pub p: f64,
}

impl NoiseChannel for Depolarizing {
    fn apply(&self, _gate: &Gate) -> Vec<(f64, Gate)> {
        let p_error = self.p / 3.0;
        vec![
            (1.0 - self.p, Gate::Rz(0, 0.0)), // Identity
            (p_error, Gate::X(0)),
            (p_error, Gate::Y(0)),
            (p_error, Gate::Z(0)),
        ]
    }

    fn name(&self) -> &str {
        "Depolarizing"
    }
}
