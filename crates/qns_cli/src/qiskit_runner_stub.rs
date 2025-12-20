/// Run a circuit using Qiskit backend
///
/// Supports: aer-ideal, aer-noisy, aer-ibm
fn cmd_run_qiskit(
    input: &PathBuf,
    backend: &str,
    ibm_backend: Option<&str>,
    shots: usize,
    format: OutputFormat,
) -> Result<()> {
    use std::process::Command;

    info!("Running with Qiskit backend: {}", backend);

    // Build Python script path
    let qns_python_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("qns_python")
        .join("python");

    // Prepare arguments for Python runner
    let mut args = vec![
        "--input".to_string(),
        input.display().to_string(),
        "--backend".to_string(),
        backend.to_string(),
        "--shots".to_string(),
        shots.to_string(),
    ];

    if let Some(ibm) = ibm_backend {
        args.push("--ibm-backend".to_string());
        args.push(ibm.to_string());
    }

    // Call Python runner script
    let output = Command::new("python")
        .arg(qns_python_path.join("cli_runner.py"))
        .args(&args)
        .output()
        .with_context(|| "Failed to execute Python runner")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Qiskit runner failed: {}", stderr);
    }

    // Parse and display output
    let stdout = String::from_utf8_lossy(&output.stdout);

    match format {
        OutputFormat::Text => {
            println!("{}", stdout);
        },
        OutputFormat::Json => {
            // Output is already JSON from Python script
            println!("{}", stdout);
        },
    }

    Ok(())
}
