use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

use spread_sim_core::model::{
    output::Output, person_info::PersonInfo, statistics::Statistics, trace::TraceEntry,
};

#[derive(Debug, Clone, Default)]
pub struct Checker {
    problems: Vec<String>,
}

impl Checker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn problems(&self) -> &[impl AsRef<str>] {
        &self.problems
    }

    pub fn has_problems(&self) -> bool {
        !self.problems.is_empty()
    }

    fn add_problem(&mut self, problem: impl Into<String>) {
        self.problems.push(problem.into());
    }

    pub fn check(&mut self, output: &Output, expected: &Output) {
        self.compare_trace(&output.trace, &expected.trace);
        self.compare_statistics(&output.statistics, &expected.statistics);
    }

    fn compare_statistics(
        &mut self,
        statistics: &HashMap<String, Vec<Statistics>>,
        expected: &HashMap<String, Vec<Statistics>>,
    ) {
        let mut query_keys: HashSet<String> = HashSet::new();
        for key in statistics.keys().chain(expected.keys()) {
            query_keys.insert(key.to_owned());
        }

        for query_key in query_keys {
            if !expected.contains_key(&query_key) {
                self.add_problem(format!("non-existent query {}", query_key));
                continue;
            }

            if !statistics.contains_key(&query_key) {
                self.add_problem(format!("no statistics for query {}", query_key));
                continue;
            }

            let entries = statistics.get(&query_key).unwrap();
            let expected_entries = expected.get(&query_key).unwrap();

            if entries.len() != expected_entries.len() {
                self.add_problem(format!(
                    "expected statistics trace of length {} but got {}",
                    expected_entries.len(),
                    entries.len()
                ));
            }

            let entries_iterator = entries.iter();
            let expected_iterator = expected_entries.iter();

            for (tick, (got_statistics, expected_statistics)) in
                zip(entries_iterator, expected_iterator).enumerate()
            {
                if !got_statistics.eq(expected_statistics) {
                    self.add_problem(format!(
                        "statistics for query `{}` incorrect in tick {} (expected: {}, got: {})",
                        query_key, tick, expected_statistics, got_statistics
                    ));
                }
            }
        }
    }

    fn compare_trace(&mut self, trace: &[TraceEntry], expected: &[TraceEntry]) {
        if trace.len() != expected.len() {
            self.add_problem(format!(
                "expected trace of length {} but got trace of length {}",
                expected.len(),
                trace.len()
            ));
        }

        let trace_iterator = trace.iter();
        let expected_iterator = expected.iter();

        for (tick, (population, expected_population)) in
            zip(trace_iterator, expected_iterator).enumerate()
        {
            self.compare_population(
                &population.population,
                &expected_population.population,
                tick,
            );
        }
    }

    fn compare_population(
        &mut self,
        population: &[PersonInfo],
        expected: &[PersonInfo],
        tick: usize,
    ) {
        if population.len() != expected.len() {
            self.add_problem(format!(
                "expected population of size {} but got population of size {} in tick {}",
                expected.len(),
                population.len(),
                tick
            ));
        }

        let population_iterator = population.iter();
        let expected_iterator = expected.iter();
        for (person_id, (person_info, expected_person_info)) in
            zip(population_iterator, expected_iterator).enumerate()
        {
            self.compare_person_info(person_info, expected_person_info, tick, person_id);
        }
    }

    fn compare_person_info(
        &mut self,
        person_info: &PersonInfo,
        expected: &PersonInfo,
        tick: usize,
        person_id: usize,
    ) {
        if !person_info.eq(expected) {
            self.add_problem(format!(
                "person information mismatch in tick {} for person with id {}",
                tick, person_id
            ));
        }
    }
}

pub fn check(output: &Output, expected: &Output) -> Checker {
    let mut checker = Checker::new();
    checker.check(output, expected);
    checker
}
