//! ZNE 설정 및 타입 정의

use serde::{Deserialize, Serialize};

/// 외삽 방법 선택
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ExtrapolationMethod {
    /// 선형 외삽: E(0) = 2*E(1) - E(2)
    #[default]
    Linear,
    /// Richardson 외삽: 다항식 피팅
    Richardson,
    /// 지수 외삽: E(λ) = a * exp(-b*λ) + c
    Exponential,
}

/// 게이트 폴딩 유형
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum FoldingType {
    /// 개별 게이트 폴딩 (G → G·G†·G)
    #[default]
    Local,
    /// 전체 회로 폴딩 (U → U·U†·U)
    Global,
}

/// ZNE 실행 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZneConfig {
    /// 외삽 방법
    pub method: ExtrapolationMethod,

    /// 스케일 팩터 목록 (노이즈 증폭 수준)
    /// 예: [1.0, 2.0, 3.0] → 원본, 2배 노이즈, 3배 노이즈
    pub scale_factors: Vec<f64>,

    /// 폴딩 유형
    pub folding_type: FoldingType,

    /// 측정 샷 수
    pub shots: usize,
}

impl Default for ZneConfig {
    fn default() -> Self {
        Self {
            method: ExtrapolationMethod::Linear,
            scale_factors: vec![1.0, 2.0, 3.0],
            folding_type: FoldingType::Local,
            shots: 1024,
        }
    }
}

impl ZneConfig {
    /// 간단한 선형 외삽 설정
    pub fn linear() -> Self {
        Self::default()
    }

    /// Richardson 외삽 설정 (더 정밀)
    pub fn richardson() -> Self {
        Self {
            method: ExtrapolationMethod::Richardson,
            scale_factors: vec![1.0, 1.5, 2.0, 2.5, 3.0],
            ..Default::default()
        }
    }

    /// 커스텀 스케일 팩터 설정
    pub fn with_scale_factors(mut self, factors: Vec<f64>) -> Self {
        self.scale_factors = factors;
        self
    }

    /// 폴딩 유형 설정
    pub fn with_folding_type(mut self, folding_type: FoldingType) -> Self {
        self.folding_type = folding_type;
        self
    }

    /// 샷 수 설정
    pub fn with_shots(mut self, shots: usize) -> Self {
        self.shots = shots;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ZneConfig::default();
        assert_eq!(config.method, ExtrapolationMethod::Linear);
        assert_eq!(config.scale_factors, vec![1.0, 2.0, 3.0]);
        assert_eq!(config.folding_type, FoldingType::Local);
    }

    #[test]
    fn test_richardson_config() {
        let config = ZneConfig::richardson();
        assert_eq!(config.method, ExtrapolationMethod::Richardson);
        assert_eq!(config.scale_factors.len(), 5);
    }
}
