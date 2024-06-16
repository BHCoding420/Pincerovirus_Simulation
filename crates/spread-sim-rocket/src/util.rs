use std::collections::HashMap;

use spread_sim_core::{
    model::{person_info::PersonInfo, statistics::Statistics},
    simulation::PersonId,
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TraceEntryWithId {
    pub population: Vec<(PersonInfo, PersonId)>,
}

impl TraceEntryWithId {
    pub fn new(population: Vec<(PersonInfo, PersonId)>) -> Self {
        Self { population }
    }
}

pub struct OutputMod {
    pub statistics: HashMap<String, Vec<Statistics>>,
    pub trace: Vec<TraceEntryWithId>,
}
impl OutputMod {
    pub fn new(trace: Vec<TraceEntryWithId>, statistics: HashMap<String, Vec<Statistics>>) -> Self {
        Self { trace, statistics }
    }
}
