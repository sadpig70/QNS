//! ZNE 에러 타입 정의

use thiserror::Error;

/// ZNE 모듈 에러 타입
#[derive(Debug, Error)]
pub enum ZneError {
    /// 스케일 팩터 부족
    #[error("Insufficient scale factors: need at least {required}, got {provided}")]
    InsufficientScaleFactors { required: usize, provided: usize },

    /// 유효하지 않은 스케일 팩터
    #[error("Invalid scale factor: {0} (must be >= 1.0)")]
    InvalidScaleFactor(f64),

    /// 외삽 실패
    #[error("Extrapolation failed: {0}")]
    ExtrapolationFailed(String),

    /// 회로 실행 에러
    #[error("Circuit execution error: {0}")]
    ExecutionError(String),

    /// 폴딩 에러
    #[error("Folding error: {0}")]
    FoldingError(String),

    /// Core 에러 래핑
    #[error("Core error: {0}")]
    CoreError(#[from] qns_core::QnsError),
}

/// ZNE 결과 타입 alias
pub type ZneResult<T> = Result<T, ZneError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ZneError::InsufficientScaleFactors {
            required: 3,
            provided: 2,
        };
        assert!(err.to_string().contains("scale factors"));
    }
}
