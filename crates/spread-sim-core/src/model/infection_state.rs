use serde::{Deserialize, Serialize};

/// Represents the state of a person.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    #[serde(rename = "healthy")]
    Susceptible,
    #[serde(rename = "infected")]
    Infected,
    #[serde(rename = "infectious")]
    Infectious,
    #[serde(rename = "recovered")]
    Recovered,
}

/// Represents the state of a person including how long it has been in that state.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct InfectionState {
    #[serde(rename = "type")]
    pub state: State,
    #[serde(rename = "since", default)]
    pub in_state_since: usize,
}

impl InfectionState {
    pub fn new(state: State, in_state_since: usize) -> Self {
        Self {
            state,
            in_state_since,
        }
    }
}
