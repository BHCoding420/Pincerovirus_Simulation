use std::collections::HashMap;

use spread_sim_core::{
    model::{
        output::Output, scenario::Scenario, statistics::Statistics, trace::TraceEntry, xy::Xy,
    },
    simulation::Person,
};

/// Auxiliary structure holding all the simulation data.
#[derive(Clone)]
struct Slug {
    scenario: Scenario,
    population: Vec<Person>,
    trace: Vec<TraceEntry>,
    statistics: HashMap<String, Vec<Statistics>>,
    positions: Vec<Xy>,
    ghosts: Vec<Xy>,
}

impl Slug {
    pub fn new(scenario: Scenario) -> Self {
        let statistics = scenario
            .queries
            .keys()
            .map(|key| (key.clone(), Vec::new()))
            .collect();
        let population = scenario
            .population
            .iter()
            .enumerate()
            .map(|(id, info)| Person::new(id.into(), info, scenario.parameters.clone()))
            .collect::<Vec<_>>();
        let positions = population.iter().map(|p| p.position).collect();
        let ghosts = Vec::with_capacity(population.len());
        let mut out = Self {
            scenario,
            population,
            trace: Vec::new(),
            statistics,
            positions,
            ghosts,
        };
        out.extend_output();
        out
    }

    fn count_persons(&self, predicate: impl Fn(&Person) -> bool) -> u64 {
        self.population
            .iter()
            .filter(|person| predicate(person))
            .count() as u64
    }

    fn extend_output(&mut self) {
        if self.scenario.trace {
            self.trace.push(TraceEntry::new(
                self.population.iter().map(Person::info).collect(),
            ))
        }
        self.extend_statistics();
    }

    fn extend_statistics(&mut self) {
        for (key, query) in &self.scenario.queries {
            let statistics = Statistics::new(
                self.count_persons(|p| p.is_susceptible() && query.area.contains(&p.position)),
                self.count_persons(|p| p.is_infected() && query.area.contains(&p.position)),
                self.count_persons(|p| p.is_infectious() && query.area.contains(&p.position)),
                self.count_persons(|p| p.is_recovered() && query.area.contains(&p.position)),
            );
            // According to the type's invariants, the entry for the key exists.
            self.statistics.get_mut(key).unwrap().push(statistics);
        }
    }

    fn tick(&mut self) {
        for (idx, person) in self.population.iter_mut().enumerate() {
            self.ghosts.push(person.position);
            person.tick(
                &self.scenario.grid(),
                &self.scenario.obstacles,
                &self.positions,
                &self.ghosts,
            );
            self.positions[idx] = person.position;
        }

        // Bust all ghosts.
        self.ghosts.clear();

        for i in 0..self.population.len() {
            for j in i + 1..self.population.len() {
                let pos_i = self.population[i].position;
                let pos_j = self.population[j].position;

                let delta_x = (pos_i.x - pos_j.x).abs();
                let delta_y = (pos_i.y - pos_j.y).abs();

                let distance = (delta_x + delta_y) as usize;
                if distance <= self.scenario.parameters.infection_radius {
                    if self.population[i].is_infectious()
                        && self.population[i].is_coughing()
                        && self.population[j].is_breathing()
                    {
                        self.population[j].infect();
                    }
                    if self.population[j].is_infectious()
                        && self.population[j].is_coughing()
                        && self.population[i].is_breathing()
                    {
                        self.population[i].infect();
                    }
                }
            }
        }

        self.extend_output();
    }

    fn into_output(self) -> Output {
        Output::new(self.scenario, self.trace, self.statistics)
    }
}

/// Let the 🐌 creep.
pub fn creep(scenario: Scenario) -> Output {
    let mut slug = Slug::new(scenario);
    for _ in 0..slug.scenario.ticks {
        slug.tick();
    }
    slug.into_output()
}
