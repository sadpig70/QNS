//! QNS Rewire - skeleton lib.rs
//! NOTE: Replace contents with full implementation as in your spec.

pub mod details;
pub mod gate_reorder;
pub mod graph;
pub mod live_rewirer;
pub mod router;
pub mod scoring;

pub use gate_reorder::{
    estimate_circuit_error, score_circuit_variant, BeamSearchConfig, CommutingPair, GateReorder,
    ReorderAnalysis, ReorderConfig,
};

pub use live_rewirer::{
    LiveRewirer, OptimizationResult, OptimizationStats, PlacementOptimizationResult, RewireConfig,
    RoutingOptimizationResult,
};
pub use router::{BasicRouter, NoiseAwareRouter, PlacementOptimizer, PlacementResult, Router};
pub use scoring::{
    // Idle-time aware functions
    calculate_qubit_schedules,
    calculate_total_idle_time,
    critical_path,
    decay_estimation,
    decay_estimation_from_noise,
    estimate_fidelity_with_hardware,
    estimate_fidelity_with_idle_tracking,
    estimate_fidelity_with_scheduling,
    gate_error_sum,
    // Hardware-aware functions (per-edge fidelity)
    gate_error_sum_with_hardware,
    QubitSchedule,
    ScoreConfig,
    ScoringError,
};
