use thiserror::Error;

#[derive(Error, Debug)]
pub enum QasmError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("QNS error: {0}")]
    QnsError(#[from] qns_core::QnsError),
}

pub type Result<T> = std::result::Result<T, QasmError>;
