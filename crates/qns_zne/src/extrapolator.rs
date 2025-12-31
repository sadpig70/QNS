//! 외삽 알고리즘 구현
//!
//! 여러 노이즈 레벨에서 측정한 기댓값을 바탕으로 제로-노이즈 기댓값 추정

use crate::error::{ZneError, ZneResult};

/// 외삽기 트레이트
pub trait Extrapolator {
    /// (scale_factor, expectation_value) 쌍에서 제로-노이즈 기댓값 추정
    fn extrapolate(&self, data: &[(f64, f64)]) -> ZneResult<f64>;
}

/// 선형 외삽
///
/// E(λ) = a + b*λ 모델 가정
/// E(0) = E(λ₁) - λ₁ * (E(λ₂) - E(λ₁)) / (λ₂ - λ₁)
pub struct LinearExtrapolator;

impl Default for LinearExtrapolator {
    fn default() -> Self {
        Self
    }
}

impl Extrapolator for LinearExtrapolator {
    fn extrapolate(&self, data: &[(f64, f64)]) -> ZneResult<f64> {
        if data.len() < 2 {
            return Err(ZneError::InsufficientScaleFactors {
                required: 2,
                provided: data.len(),
            });
        }

        // 최소제곱법으로 선형 피팅
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-15 {
            return Err(ZneError::ExtrapolationFailed(
                "Degenerate data points".to_string(),
            ));
        }

        // y = a + b*x
        let b = (n * sum_xy - sum_x * sum_y) / denom;
        let a = (sum_y - b * sum_x) / n;

        // E(0) = a
        Ok(a)
    }
}

/// Richardson 외삽
///
/// 다항식 피팅: E(λ) = a₀ + a₁λ + a₂λ² + ...
/// 데이터 개수에 따라 다항식 차수 결정
pub struct RichardsonExtrapolator {
    /// 최대 다항식 차수
    pub max_order: usize,
}

impl Default for RichardsonExtrapolator {
    fn default() -> Self {
        Self { max_order: 3 }
    }
}

impl RichardsonExtrapolator {
    /// 최대 차수 설정
    pub fn with_max_order(mut self, order: usize) -> Self {
        self.max_order = order;
        self
    }
}

impl Extrapolator for RichardsonExtrapolator {
    fn extrapolate(&self, data: &[(f64, f64)]) -> ZneResult<f64> {
        if data.len() < 2 {
            return Err(ZneError::InsufficientScaleFactors {
                required: 2,
                provided: data.len(),
            });
        }

        // 다항식 차수: min(데이터 개수 - 1, max_order)
        let order = std::cmp::min(data.len() - 1, self.max_order);

        // Vandermonde 행렬을 이용한 다항식 피팅 (간단한 구현)
        // 여기서는 order=1이면 선형, order=2이면 2차 등
        // 정확한 구현을 위해서는 행렬 역행렬 또는 QR 분해 필요

        // 간단한 경우: order=1 → 선형 외삽으로 대체
        if order == 1 {
            return LinearExtrapolator.extrapolate(data);
        }

        // order=2: 2차 다항식 (3개 데이터 포인트 필요)
        if order >= 2 && data.len() >= 3 {
            let (x1, y1) = data[0];
            let (x2, y2) = data[1];
            let (x3, y3) = data[2];

            // Lagrange 보간
            // L₀(0) = (0-x₂)(0-x₃) / (x₁-x₂)(x₁-x₃)
            // L₁(0) = (0-x₁)(0-x₃) / (x₂-x₁)(x₂-x₃)
            // L₂(0) = (0-x₁)(0-x₂) / (x₃-x₁)(x₃-x₂)

            let l0 = (x2 * x3) / ((x1 - x2) * (x1 - x3));
            let l1 = (x1 * x3) / ((x2 - x1) * (x2 - x3));
            let l2 = (x1 * x2) / ((x3 - x1) * (x3 - x2));

            let e0 = y1 * l0 + y2 * l1 + y3 * l2;
            return Ok(e0);
        }

        // 폴백: 선형 외삽
        LinearExtrapolator.extrapolate(data)
    }
}

/// 지수 외삽 (간단한 구현)
///
/// E(λ) = a * exp(-b*λ) + c 모델
/// 비선형 최적화 필요 → 근사적 구현
pub struct ExponentialExtrapolator;

impl Default for ExponentialExtrapolator {
    fn default() -> Self {
        Self
    }
}

impl Extrapolator for ExponentialExtrapolator {
    fn extrapolate(&self, data: &[(f64, f64)]) -> ZneResult<f64> {
        if data.len() < 3 {
            return Err(ZneError::InsufficientScaleFactors {
                required: 3,
                provided: data.len(),
            });
        }

        // 간단한 근사: log 변환 후 선형 피팅
        // E - E_∞ ≈ A * exp(-b*λ)
        // log(E - E_∞) ≈ log(A) - b*λ

        // E_∞ 추정: 가장 높은 노이즈에서의 값 (수렴값)
        let e_inf = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);

        // 로그 변환 (양수 보장)
        let log_data: Vec<(f64, f64)> = data
            .iter()
            .filter_map(|(x, y)| {
                let diff = y - e_inf + 0.01; // 작은 오프셋 추가
                if diff > 0.0 {
                    Some((*x, diff.ln()))
                } else {
                    None
                }
            })
            .collect();

        if log_data.len() < 2 {
            // 로그 변환 실패 시 선형 외삽 폴백
            return LinearExtrapolator.extrapolate(data);
        }

        // 선형 피팅
        let linear = LinearExtrapolator.extrapolate(&log_data)?;

        // exp 역변환: E(0) = exp(linear_result) + E_∞
        Ok(linear.exp() + e_inf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_extrapolator() {
        let extrap = LinearExtrapolator;

        // y = 1 - 0.1x 데이터
        let data = vec![(1.0, 0.9), (2.0, 0.8), (3.0, 0.7)];
        let e0 = extrap.extrapolate(&data).unwrap();

        // E(0) ≈ 1.0
        assert!((e0 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_richardson_extrapolator() {
        let extrap = RichardsonExtrapolator::default();

        // 2차 데이터: y = 1 - 0.1x + 0.01x²
        let data = vec![(1.0, 0.91), (2.0, 0.84), (3.0, 0.79)];
        let e0 = extrap.extrapolate(&data).unwrap();

        // E(0) ≈ 1.0
        assert!((e0 - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_insufficient_data() {
        let extrap = LinearExtrapolator;
        let data = vec![(1.0, 0.9)];

        assert!(extrap.extrapolate(&data).is_err());
    }
}
