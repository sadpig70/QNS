//! 게이트 폴딩 (노이즈 증폭) 구현
//!
//! Local Folding: G → G · G† · G
//! 이를 통해 게이트 에러를 인위적으로 증폭시킴

use qns_core::prelude::*;

use crate::error::{ZneError, ZneResult};

/// 회로 폴딩 트레이트
pub trait CircuitFolder {
    /// 회로의 노이즈를 scale_factor 배로 증폭
    fn fold(&self, circuit: &CircuitGenome, scale_factor: f64) -> ZneResult<CircuitGenome>;
}

/// Local Folding 구현
///
/// 개별 게이트를 G → G·G†·G 형태로 확장하여 노이즈 증폭
/// scale_factor = 1 → 원본
/// scale_factor = 3 → 모든 게이트 1회 폴딩 (G → G·G†·G)
/// scale_factor = 5 → 모든 게이트 2회 폴딩 (G → G·G†·G·G†·G)
pub struct LocalFolder {
    /// 2-qubit 게이트만 폴딩할지 여부
    pub fold_only_2q: bool,
}

impl Default for LocalFolder {
    fn default() -> Self {
        Self { fold_only_2q: true }
    }
}

impl LocalFolder {
    /// 새 LocalFolder 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 2-qubit 게이트만 폴딩 설정
    pub fn with_fold_only_2q(mut self, only_2q: bool) -> Self {
        self.fold_only_2q = only_2q;
        self
    }

    /// 게이트의 adjoint (†) 반환
    fn adjoint(&self, gate: &Gate) -> Gate {
        match gate {
            // Hermitian 게이트: 자신이 adjoint
            Gate::H(q) => Gate::H(*q),
            Gate::X(q) => Gate::X(*q),
            Gate::Y(q) => Gate::Y(*q),
            Gate::Z(q) => Gate::Z(*q),
            Gate::CNOT(c, t) => Gate::CNOT(*c, *t),
            Gate::CZ(c, t) => Gate::CZ(*c, *t),
            Gate::SWAP(a, b) => Gate::SWAP(*a, *b),

            // S† 및 T† (역방향 회전)
            Gate::S(q) => Gate::Rz(*q, -std::f64::consts::FRAC_PI_2),
            Gate::T(q) => Gate::Rz(*q, -std::f64::consts::FRAC_PI_4),

            // 파라미터 게이트: 부호 반전
            Gate::Rx(q, theta) => Gate::Rx(*q, -theta),
            Gate::Ry(q, theta) => Gate::Ry(*q, -theta),
            Gate::Rz(q, theta) => Gate::Rz(*q, -theta),

            // Measure: 폴딩 불가
            Gate::Measure(q) => Gate::Measure(*q),
        }
    }

    /// 게이트가 2-qubit인지 확인
    fn is_2q_gate(&self, gate: &Gate) -> bool {
        matches!(gate, Gate::CNOT(_, _) | Gate::CZ(_, _) | Gate::SWAP(_, _))
    }

    /// 단일 게이트 폴딩 (num_folds 회)
    fn fold_gate(&self, gate: &Gate, num_folds: usize) -> Vec<Gate> {
        if matches!(gate, Gate::Measure(_)) {
            // Measure는 폴딩하지 않음
            return vec![gate.clone()];
        }

        if self.fold_only_2q && !self.is_2q_gate(gate) {
            // 1-qubit 게이트는 폴딩하지 않음 (옵션)
            return vec![gate.clone()];
        }

        let mut result = vec![gate.clone()];

        for _ in 0..num_folds {
            // G → G · G† · G
            result.push(self.adjoint(gate));
            result.push(gate.clone());
        }

        result
    }
}

impl CircuitFolder for LocalFolder {
    fn fold(&self, circuit: &CircuitGenome, scale_factor: f64) -> ZneResult<CircuitGenome> {
        if scale_factor < 1.0 {
            return Err(ZneError::InvalidScaleFactor(scale_factor));
        }

        // scale_factor가 1에 가까우면 원본 반환
        if (scale_factor - 1.0).abs() < 1e-9 {
            return Ok(circuit.clone());
        }

        // 폴딩 횟수 계산: scale_factor = 1 + 2*num_folds
        // num_folds = (scale_factor - 1) / 2
        let num_folds = ((scale_factor - 1.0) / 2.0).round() as usize;

        let mut folded = CircuitGenome::new(circuit.num_qubits);

        for gate in &circuit.gates {
            let expanded_gates = self.fold_gate(gate, num_folds);
            for g in expanded_gates {
                folded.add_gate(g)?;
            }
        }

        Ok(folded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_scale_1() {
        let folder = LocalFolder::new();
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let folded = folder.fold(&circuit, 1.0).unwrap();
        assert_eq!(folded.gates.len(), 2);
    }

    #[test]
    fn test_fold_scale_3() {
        let folder = LocalFolder::new().with_fold_only_2q(true);
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let folded = folder.fold(&circuit, 3.0).unwrap();
        // H는 폴딩되지 않음 (1-qubit)
        // CNOT → CNOT · CNOT† · CNOT (3개)
        assert_eq!(folded.gates.len(), 1 + 3);
    }

    #[test]
    fn test_adjoint_hermitian() {
        let folder = LocalFolder::new();

        // Hermitian 게이트: 자신이 adjoint
        assert_eq!(folder.adjoint(&Gate::H(0)), Gate::H(0));
        assert_eq!(folder.adjoint(&Gate::X(1)), Gate::X(1));
        assert_eq!(folder.adjoint(&Gate::CNOT(0, 1)), Gate::CNOT(0, 1));
    }

    #[test]
    fn test_adjoint_rotation() {
        let folder = LocalFolder::new();

        // Rotation: 부호 반전
        if let Gate::Rx(_, theta) = folder.adjoint(&Gate::Rx(0, 0.5)) {
            assert!((theta + 0.5).abs() < 1e-9);
        } else {
            panic!("Expected Rx");
        }
    }
}
