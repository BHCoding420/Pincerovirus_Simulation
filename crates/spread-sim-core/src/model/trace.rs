use serde::{Deserialize, Serialize};

use super::person_info::PersonInfo;

/// Represents the population at some point in time.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct TraceEntry {
    /// The population of the entry.
    pub population: Vec<PersonInfo>,
}

impl TraceEntry {
    pub fn new(population: Vec<PersonInfo>) -> Self {
        Self { population }
    }
}
