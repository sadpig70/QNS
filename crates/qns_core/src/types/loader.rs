use super::{CrosstalkMatrix, HardwareProfile};
use crate::error::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Loader for hardware profiles and crosstalk data from external sources.
pub struct CrosstalkLoader;

impl CrosstalkLoader {
    /// Load HardwareProfile from a JSON file.
    pub fn load_profile<P: AsRef<Path>>(path: P) -> Result<HardwareProfile> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let profile = serde_json::from_reader(reader)?;
        Ok(profile)
    }

    /// Load CrosstalkMatrix from a JSON file.
    /// Expected format: {"(0,1)": 0.01, ...} or list of interactions.
    pub fn load_matrix<P: AsRef<Path>>(path: P) -> Result<CrosstalkMatrix> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let matrix = serde_json::from_reader(reader)?;
        Ok(matrix)
    }
}
