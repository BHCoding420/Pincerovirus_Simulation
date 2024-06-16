use serde::{Deserialize, Serialize};

/// Container for the simulation parameters.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// The threshold determining how often a person coughs.
    pub cough_threshold: usize,
    /// The threshold determining how often a person breaths.
    pub breath_threshold: usize,
    /// The divisor determining how likely a person is to accelerate.
    pub acceleration_divisor: usize,
    /// The number of ticks a person is infectious before recovering.
    pub recovery_time: usize,
    /// The maximum (non-diagonal) distance the infection can spread directly.
    pub infection_radius: usize,
    /// The number of ticks a person is infected before becoming infectious.
    pub incubation_time: usize,
}

impl Parameters {
    pub fn new(
        cough_threshold: usize,
        breath_threshold: usize,
        acceleration_divisor: usize,
        recovery_time: usize,
        infection_radius: usize,
        incubation_time: usize,
    ) -> Self {
        Self {
            cough_threshold,
            breath_threshold,
            acceleration_divisor,
            recovery_time,
            infection_radius,
            incubation_time,
        }
    }
}
