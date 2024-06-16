use std::{collections::HashMap, error::Error, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::{scenario::Scenario, statistics::Statistics, trace::TraceEntry};

/// The output to be computed by the simulator.
#[derive(Serialize, Deserialize, Clone)]
pub struct Output {
    /// The simulation scenario.
    pub scenario: Scenario,
    /// The trace.
    pub trace: Vec<TraceEntry>,
    /// The collected statistics.
    #[serde(rename = "stats")]
    pub statistics: HashMap<String, Vec<Statistics>>,
}

impl Output {
    pub fn new(
        scenario: Scenario,
        trace: Vec<TraceEntry>,
        statistics: HashMap<String, Vec<Statistics>>,
    ) -> Self {
        Self {
            scenario,
            trace,
            statistics,
        }
    }
}

/// Error loading or saving an [`Output`].
#[derive(Error, Debug)]
#[error(transparent)]
pub struct OutputError(Box<dyn Error>);

impl OutputError {
    /// Turns any error into an [`OutputError`].
    fn new(error: impl 'static + Error) -> Self {
        Self(Box::new(error))
    }
}

/// Tries to save a simulation output to the provided path.
pub fn save(output: &Output, path: impl AsRef<Path>) -> Result<(), OutputError> {
    let src = serde_json::to_string(&output).map_err(OutputError::new)?;
    std::fs::write(path, src).map_err(OutputError::new)
}

/// Tries to parse a simulation output from the provided string.
pub fn from_str(src: &str) -> Result<Output, OutputError> {
    serde_json::from_str(src).map_err(OutputError::new)
}

/// Tries to load a scenario from the provided path.
pub fn load(path: impl AsRef<Path>) -> Result<Output, OutputError> {
    from_str(&std::fs::read_to_string(path).map_err(OutputError::new)?)
}
