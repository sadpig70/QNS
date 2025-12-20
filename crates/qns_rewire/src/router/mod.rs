use qns_core::{CircuitGenome, HardwareProfile, QnsError};

pub trait Router {
    fn route(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> Result<CircuitGenome, QnsError>;
}

pub mod basic;
pub mod noise_aware;
pub mod placement;

pub use basic::BasicRouter;
pub use noise_aware::NoiseAwareRouter;
pub use placement::{PlacementOptimizer, PlacementResult};
