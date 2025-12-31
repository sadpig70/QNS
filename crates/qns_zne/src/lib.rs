//! # QNS ZNE (Zero-Noise Extrapolation)
//!
//! 노이즈 외삽 기반 오류 완화 모듈.
//!
//! ## 주요 기능
//!
//! - **LocalFolding**: 게이트 수준 노이즈 증폭 (CNOT → CNOT-CNOT†-CNOT)
//! - **Extrapolation**: Linear, Richardson, Exponential 외삽 알고리즘
//! - **ZneExecutor**: 전체 ZNE 파이프라인 실행
//!
//! ## 사용 예시
//!
//! ```rust,ignore
//! use qns_zne::{ZneConfig, ZneExecutor, ExtrapolationMethod};
//! use qns_core::CircuitGenome;
//!
//! let circuit = CircuitGenome::new(2);
//! let config = ZneConfig::default();
//! let executor = ZneExecutor::new(config);
//!
//! // 노이즈 외삽 실행
//! let zero_noise_expectation = executor.execute(&circuit)?;
//! ```

pub mod extrapolator;
pub mod folding;

mod config;
mod error;
mod executor;

pub use config::{ExtrapolationMethod, FoldingType, ZneConfig};
pub use error::{ZneError, ZneResult};
pub use executor::{CircuitExecutor, FidelityEstimator, ZneExecutionResult, ZneExecutor};
pub use extrapolator::{Extrapolator, LinearExtrapolator, RichardsonExtrapolator};
pub use folding::{CircuitFolder, LocalFolder};
