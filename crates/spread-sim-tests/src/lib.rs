use std::{
    panic::AssertUnwindSafe,
    path::Path,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use spread_sim_core::{
    model::{
        output::{self, Output},
        scenario::{self, Scenario},
    },
    validator::{DummyValidator, Validator},
};

#[cfg(test)]
mod tests;

pub mod checker;
pub mod scenarios;

/// A test scenario.
#[derive(Debug, Clone, Copy)]
pub struct TestScenario {
    pub root: &'static str,
    pub name: &'static str,
}

impl TestScenario {
    pub fn root_path(&self) -> &Path {
        Path::new(&self.root)
    }

    pub fn load_scenario(&self) -> Scenario {
        let path = self.root_path().join(self.name).with_extension("json");
        scenario::load(&path)
            .map_err(|_| panic!("Unable to load scenario from {path:?}."))
            .unwrap()
    }

    pub fn load_output(&self) -> Output {
        let path = self
            .root_path()
            .join(self.name)
            .with_extension("result.json");
        output::load(path).unwrap()
    }

    pub fn test_case(&self) -> TestCase {
        TestCase {
            scenario: self.load_scenario(),
            output: self.load_output(),
            validator: Arc::new(DummyValidator),
            timeout: Duration::from_secs(60),
            padding: 10,
            starship: false,
        }
    }
}

pub struct TestCase {
    pub scenario: Scenario,
    output: Output,
    validator: Arc<dyn Validator>,
    timeout: Duration,
    padding: usize,
    starship: bool,
}

impl TestCase {
    pub fn with_validator(mut self, validator: Arc<dyn Validator>) -> Self {
        self.validator = validator;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_starship(mut self, starship: bool) -> Self {
        self.starship = starship;
        self
    }

    pub fn creep(self) {
        self.run(spread_sim_slug::creep)
    }

    pub fn launch(self) {
        let padding = self.padding;
        let starship = self.starship;
        let validator = self.validator.clone();
        self.run(move |scenario| {
            spread_sim_rocket::launch(scenario, padding, validator, starship).unwrap()
        })
    }

    pub fn run(self, simulate: impl 'static + Send + FnOnce(Scenario) -> Output) {
        let (tx, rx) = mpsc::channel();
        let start = Instant::now();
        thread::spawn(move || {
            let _ = tx.send(std::panic::catch_unwind(AssertUnwindSafe(move || {
                simulate(self.scenario)
            })));
        });
        let duration = Instant::now() - start;
        let Ok(result) = rx.recv_timeout(self.timeout) else {
            panic!("Timeout after {} seconds!", duration.as_secs_f32());
        };
        match result {
            Ok(output) => {
                let checker = checker::check(&output, &self.output);
                if checker.has_problems() {
                    for problem in checker.problems() {
                        eprintln!("Problem: {}", problem.as_ref())
                    }
                    panic!("Output does not match expected output!");
                }
            }
            Err(error) => {
                let msg = error
                    .as_ref()
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| error.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("Panic payload is not a string.");
                panic!("Simulation panicked with: {msg}");
            }
        }
    }
}

/// Macro for defining test scenarios.
#[macro_export]
macro_rules! test_scenario {
    ($name:ident, $category:literal, $scenario:literal) => {
        pub static $name: $crate::TestScenario = $crate::TestScenario {
            root: concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios/", $category),
            name: $scenario,
        };
    };
}

/// Macro for defining public test scenarios.
#[macro_export]
macro_rules! test_scenario_public {
    ($name:ident, $scenario:literal) => {
        $crate::test_scenario!($name, "public", $scenario);
    };
}
