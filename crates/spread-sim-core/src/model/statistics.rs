use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents SI²R-statistics at some point in time.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Statistics {
    /// The number of susceptible persons.
    pub susceptible: u64,
    /// The number of infected persons.
    pub infected: u64,
    /// The number of infectious persons.
    pub infectious: u64,
    /// The number of recovered persons.
    pub recovered: u64,
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics({}, {}, {}, {})",
            self.susceptible, self.infected, self.infectious, self.recovered
        )
    }
}

impl Statistics {
    pub fn new(susceptible: u64, infected: u64, infectious: u64, recovered: u64) -> Self {
        Self {
            susceptible,
            infected,
            infectious,
            recovered,
        }
    }

    /// Adds the provided statistics to `self`.
    pub fn add(&mut self, other: &Statistics) {
        self.susceptible += other.susceptible;
        self.infected += other.infected;
        self.infectious += other.infectious;
        self.recovered += other.recovered;
    }
}
