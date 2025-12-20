use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoiseError {
    #[error("Invalid probability: {0}")]
    InvalidProbability(f64),
}
