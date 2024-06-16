use std::{collections::HashMap, error::Error, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    parameters::Parameters, partition::Partition, person_info::PersonInfo, query::Query,
    rectangle::Rectangle, xy::Xy,
};

/// Represents a simulation scenario.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scenario {
    /// The name of the scenario.
    pub name: String,
    /// The simulation parameters of the scenario.
    pub parameters: Arc<Parameters>,
    /// The amount of ticks to simulate.
    pub ticks: usize,
    /// The size of the grid of the simulation.
    #[serde(rename = "gridSize")]
    pub grid_size: Xy,
    /// Indicates whether a full trace should be captured.
    pub trace: bool,
    /// The partition of the grid into patches.
    pub partition: Partition,
    /// The obstacles on the grid.
    pub obstacles: Vec<Rectangle>,
    /// The statistic queries to compute.
    #[serde(rename = "statQueries")]
    pub queries: HashMap<String, Query>,
    /// The population of the scenario.
    pub population: Vec<PersonInfo>,
}

impl Scenario {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        parameters: Arc<Parameters>,
        ticks: usize,
        grid_size: Xy,
        trace: bool,
        partition: Partition,
        obstacles: Vec<Rectangle>,
        queries: HashMap<String, Query>,
        population: Vec<PersonInfo>,
    ) -> Self {
        Self {
            name,
            parameters,
            ticks,
            grid_size,
            trace,
            partition,
            obstacles,
            queries,
            population,
        }
    }

    /// Returns a [`Rectangle`] representing the grid.
    pub fn grid(&self) -> Rectangle {
        Rectangle::new(Xy::zero(), self.grid_size)
    }

    /// Returns the number of patches.
    pub fn number_of_patches(&self) -> usize {
        (self.partition.x.len() + 1) * (self.partition.y.len() + 1)
    }

    /// Indicates whether a cell is placed on an obstacle.
    pub fn on_obstacle(&self, cell: &Xy) -> bool {
        self.obstacles.iter().any(|x: &Rectangle| x.contains(cell))
    }
}

/// Error parsing or loading a scenario.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct ScenarioError(Box<dyn Error>);

impl ScenarioError {
    /// Turns any error into a [`ScenarioError`].
    fn new(error: impl 'static + Error) -> Self {
        Self(Box::new(error))
    }
}

/// Tries to parse a scenario from the provided string.
pub fn from_str(src: &str) -> Result<Scenario, ScenarioError> {
    serde_json::from_str(src).map_err(ScenarioError::new)
}

/// Tries to load a scenario from the provided path.
pub fn load(path: impl AsRef<Path>) -> Result<Scenario, ScenarioError> {
    from_str(&std::fs::read_to_string(path).map_err(ScenarioError::new)?)
}
