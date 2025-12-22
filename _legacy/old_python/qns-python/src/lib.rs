use pyo3::prelude::*;

mod circuit;
mod simulator;
mod types;

use circuit::QuantumCircuit;
use simulator::Simulator;

/// QNS Python Module
#[pymodule]
fn _qns(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<QuantumCircuit>()?;
    m.add_class::<Simulator>()?;
    Ok(())
}
