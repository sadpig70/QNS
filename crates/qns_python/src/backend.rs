//! qns_python backend.rs (skeleton)
//! NOTE: Paste full PyExecutionResult, PySimulatorBackend, PyMockBackend implementation here.

use pyo3::prelude::*;
use std::collections::HashMap;
use qns_core::backend::ExecutionResult;

#[pyclass(name = "ExecutionResult")]
#[derive(Clone)]
pub struct PyExecutionResult {
    pub inner: ExecutionResult,
}
