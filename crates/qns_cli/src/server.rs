use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
pub struct SimulateRequest {
    qasm: String,
    topology: Option<String>,
    noise_model: Option<String>,
}

#[derive(Serialize)]
pub struct SimulateResponse {
    results: std::collections::HashMap<String, usize>,
    execution_time_ms: f64,
    qubit_count: usize,
    gate_count: usize,
}

pub async fn start_server() {
    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/simulate", post(handle_simulate))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_simulate(Json(payload): Json<SimulateRequest>) -> Json<SimulateResponse> {
    // Note: This runs synchronously for now, blocking the async thread.
    // Ideally use spawn_blocking, but for MVP it's fine.

    let start = std::time::Instant::now();

    // We need a version of pipeline that takes string input
    match run_simulation_from_string(
        &payload.qasm,
        payload.topology.as_deref().unwrap_or("linear"),
        payload.noise_model.as_deref(),
    ) {
        Ok((results, q_count, g_count)) => {
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            Json(SimulateResponse {
                results,
                execution_time_ms: duration,
                qubit_count: q_count,
                gate_count: g_count,
            })
        },
        Err(e) => {
            // Simple error handling: return empty results with error logged (or change response type)
            eprintln!("Simulation error: {}", e);
            Json(SimulateResponse {
                results: std::collections::HashMap::new(),
                execution_time_ms: 0.0,
                qubit_count: 0,
                gate_count: 0,
            })
        },
    }
}

// Refactored pipeline logic
fn run_simulation_from_string(
    qasm_content: &str,
    topology: &str,
    noise_config: Option<&str>,
) -> anyhow::Result<(std::collections::HashMap<String, usize>, usize, usize)> {
    use qns_core::HardwareProfile;
    use qns_noise::{BitFlip, Depolarizing, NoiseChannel, PhaseFlip};
    use qns_qasm::parse_qasm;
    use qns_rewire::{BasicRouter, Router};
    use qns_tensor::TensorNetwork;

    // 1. Parse
    let circuit = parse_qasm(qasm_content)?;
    let q_count = circuit.num_qubits;

    // 2. Route
    let hardware = match topology {
        "grid" => HardwareProfile::grid("grid", 5, 5), // Example
        _ => HardwareProfile::linear("linear", std::cmp::max(q_count, 20)),
    };

    let router = BasicRouter;
    let routed_circuit = router.route(&circuit, &hardware)?;
    let g_count = routed_circuit.gates.len();

    // 3. Simulate
    let mut tn = TensorNetwork::new(routed_circuit.num_qubits, 4);

    // Configure Noise
    if let Some(config) = noise_config {
        let parts: Vec<&str> = config.split(':').collect();
        if parts.len() == 2 {
            let name = parts[0];
            let prob: f64 = parts[1].parse().unwrap_or(0.0);

            let channel: Box<dyn NoiseChannel> = match name {
                "depolarizing" => Box::new(Depolarizing { p: prob }),
                "bitflip" => Box::new(BitFlip { p: prob }),
                // Add more channels here
                _ => Box::new(Depolarizing { p: 0.0 }), // Default or error
            };
            tn = tn.with_noise(channel);
        }
    }

    for gate in &routed_circuit.gates {
        if let qns_core::types::Gate::Measure(_) = gate {
            continue;
        }
        tn.apply_gate(gate)?;
    }

    let counts = tn.measure(1000)?;
    Ok((counts, q_count, g_count))
}
