//! ZNE 실행기 - 전체 파이프라인 구현

use qns_core::prelude::*;

use crate::config::{ExtrapolationMethod, ZneConfig};
use crate::error::{ZneError, ZneResult};
use crate::extrapolator::{
    ExponentialExtrapolator, Extrapolator, LinearExtrapolator, RichardsonExtrapolator,
};
use crate::folding::{CircuitFolder, LocalFolder};

/// 회로 실행기 트레이트
///
/// 외부에서 주입하여 실제 시뮬레이터나 하드웨어로 실행
pub trait CircuitExecutor {
    /// 회로 실행 후 기댓값 반환
    fn execute(&self, circuit: &CircuitGenome, shots: usize) -> ZneResult<f64>;
}

/// 간단한 Fidelity 기반 실행기 (시뮬레이션용)
pub struct FidelityEstimator {
    /// 기준 노이즈 레벨 (1-qubit 게이트 에러율)
    pub base_error_1q: f64,
    /// 기준 노이즈 레벨 (2-qubit 게이트 에러율)
    pub base_error_2q: f64,
}

impl Default for FidelityEstimator {
    fn default() -> Self {
        Self {
            base_error_1q: 0.001,
            base_error_2q: 0.01,
        }
    }
}

impl CircuitExecutor for FidelityEstimator {
    fn execute(&self, circuit: &CircuitGenome, _shots: usize) -> ZneResult<f64> {
        // 간단한 Fidelity 추정: (1-ε)^n
        let mut fidelity = 1.0;

        for gate in &circuit.gates {
            let error = match gate {
                Gate::CNOT(_, _) | Gate::CZ(_, _) | Gate::SWAP(_, _) => self.base_error_2q,
                Gate::Measure(_) => 0.0,
                _ => self.base_error_1q,
            };
            fidelity *= 1.0 - error;
        }

        Ok(fidelity)
    }
}

/// ZNE 실행기
///
/// 전체 Zero-Noise Extrapolation 파이프라인 관리
pub struct ZneExecutor<E: CircuitExecutor> {
    /// ZNE 설정
    pub config: ZneConfig,
    /// 회로 실행기
    executor: E,
    /// 폴딩 구현체
    folder: Box<dyn CircuitFolder + Send + Sync>,
}

impl<E: CircuitExecutor> ZneExecutor<E> {
    /// 새 ZNE 실행기 생성
    pub fn new(config: ZneConfig, executor: E) -> Self {
        Self {
            config,
            executor,
            folder: Box::new(LocalFolder::default()),
        }
    }

    /// 커스텀 폴더 설정
    pub fn with_folder<F: CircuitFolder + Send + Sync + 'static>(mut self, folder: F) -> Self {
        self.folder = Box::new(folder);
        self
    }

    /// ZNE 실행 - 제로-노이즈 기댓값 추정
    pub fn execute(&self, circuit: &CircuitGenome) -> ZneResult<ZneExecutionResult> {
        // 1. 스케일 팩터 검증
        if self.config.scale_factors.len() < 2 {
            return Err(ZneError::InsufficientScaleFactors {
                required: 2,
                provided: self.config.scale_factors.len(),
            });
        }

        // 2. 각 스케일 팩터에 대해 회로 폴딩 및 실행
        let mut data_points: Vec<(f64, f64)> = Vec::new();
        let mut scaled_circuits: Vec<(f64, CircuitGenome)> = Vec::new();

        for &scale in &self.config.scale_factors {
            // 회로 폴딩
            let folded = self.folder.fold(circuit, scale)?;

            // 실행
            let expectation = self.executor.execute(&folded, self.config.shots)?;

            data_points.push((scale, expectation));
            scaled_circuits.push((scale, folded));
        }

        // 3. 외삽
        let zero_noise_value = self.extrapolate(&data_points)?;

        Ok(ZneExecutionResult {
            zero_noise_value,
            data_points,
            method: self.config.method,
        })
    }

    /// 외삽 수행
    fn extrapolate(&self, data: &[(f64, f64)]) -> ZneResult<f64> {
        match self.config.method {
            ExtrapolationMethod::Linear => LinearExtrapolator.extrapolate(data),
            ExtrapolationMethod::Richardson => RichardsonExtrapolator::default().extrapolate(data),
            ExtrapolationMethod::Exponential => ExponentialExtrapolator.extrapolate(data),
        }
    }
}

/// ZNE 실행 결과
#[derive(Debug, Clone)]
pub struct ZneExecutionResult {
    /// 제로-노이즈 외삽 기댓값
    pub zero_noise_value: f64,
    /// 각 스케일 팩터에서의 측정값
    pub data_points: Vec<(f64, f64)>,
    /// 사용된 외삽 방법
    pub method: ExtrapolationMethod,
}

impl ZneExecutionResult {
    /// 개선율 계산 (noisy vs zero-noise)
    pub fn improvement(&self) -> f64 {
        if let Some((_, noisy_value)) = self.data_points.first() {
            if *noisy_value > 0.0 {
                return (self.zero_noise_value - noisy_value) / noisy_value * 100.0;
            }
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zne_executor_linear() {
        let config = ZneConfig::linear();
        let executor = FidelityEstimator::default();
        let zne = ZneExecutor::new(config, executor);

        // 간단한 Bell 회로
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let result = zne.execute(&circuit).unwrap();

        // 제로-노이즈 값이 원본보다 높아야 함
        assert!(result.zero_noise_value > 0.0);
        assert!(result.data_points.len() >= 2);
    }

    #[test]
    fn test_improvement_calculation() {
        let result = ZneExecutionResult {
            zero_noise_value: 0.95,
            data_points: vec![(1.0, 0.90), (2.0, 0.85), (3.0, 0.80)],
            method: ExtrapolationMethod::Linear,
        };

        let improvement = result.improvement();
        // (0.95 - 0.90) / 0.90 * 100 ≈ 5.56%
        assert!(improvement > 5.0 && improvement < 6.0);
    }
}
