use std::sync::Arc;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

use super::{direction::Direction, infection_state::InfectionState, xy::Xy};

/// Represents a person on the grid.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PersonInfo {
    /// The name of the person.
    pub name: Arc<String>,
    /// The position of the person.
    #[serde(rename = "pos")]
    pub position: Xy,
    /// The seed (used for the RNG) of the person.
    #[serde(rename = "rngState", with = "Base64Standard")]
    pub seed: Vec<u8>,
    /// The infection state of the person.
    #[serde(rename = "infectionState")]
    pub infection_state: InfectionState,
    /// The direction the person is moving in.
    pub direction: Direction,
}

impl PersonInfo {
    pub fn new(
        name: Arc<String>,
        position: Xy,
        seed: Vec<u8>,
        infection_state: InfectionState,
        direction: Direction,
    ) -> Self {
        Self {
            name,
            position,
            seed,
            infection_state,
            direction,
        }
    }
}
