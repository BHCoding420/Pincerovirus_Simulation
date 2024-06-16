use std::sync::Arc;

use crate::model::{
    direction::Direction,
    infection_state::{InfectionState, State},
    parameters::Parameters,
    person_info::PersonInfo,
    rectangle::Rectangle,
    xy::Xy,
};

/// Uniquely identifies a person in a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersonId(usize);

impl From<usize> for PersonId {
    fn from(value: usize) -> Self {
        PersonId(value)
    }
}

impl From<PersonId> for usize {
    fn from(value: PersonId) -> Self {
        value.0
    }
}

/// A person.
#[derive(Debug, Clone)]
pub struct Person {
    /// The id of the person.
    pub id: PersonId,
    /// The parameters of the simulation.
    pub parameters: Arc<Parameters>,
    /// The name of the person.
    pub name: Arc<String>,
    /// The position of the person.
    pub position: Xy,
    /// The direction the person is heading to.
    pub direction: Direction,
    /// The infection state of the person.
    infection_state: InfectionState,
    /// The internal state of the random number generator.
    rng: Rng,
}

impl Person {
    pub fn new(id: PersonId, info: &PersonInfo, parameters: Arc<Parameters>) -> Self {
        Self {
            id,
            parameters: parameters.clone(),
            name: info.name.clone(),
            position: info.position,
            direction: info.direction,
            infection_state: info.infection_state,
            rng: Rng::new(&info.seed, parameters),
        }
    }

    pub fn state(&self) -> State {
        self.infection_state.state
    }

    pub fn set_state(&mut self, state: State) {
        self.infection_state.state = state;
        self.infection_state.in_state_since = 0;
    }

    pub fn in_state_since(&self) -> usize {
        self.infection_state.in_state_since
    }

    pub fn is_susceptible(&self) -> bool {
        self.state() == State::Susceptible
    }

    pub fn is_infected(&self) -> bool {
        self.state() == State::Infected
    }

    pub fn is_infectious(&self) -> bool {
        self.state() == State::Infectious
    }

    pub fn is_recovered(&self) -> bool {
        self.state() == State::Recovered
    }

    pub fn is_breathing(&self) -> bool {
        self.rng.is_breathing()
    }

    pub fn is_coughing(&self) -> bool {
        self.rng.is_coughing()
    }

    pub fn infect(&mut self) {
        if self.is_susceptible() {
            self.set_state(State::Infected);
        }
    }

    pub fn info(&self) -> PersonInfo {
        PersonInfo::new(
            self.name.clone(),
            self.position,
            self.rng.digest().clone(),
            self.infection_state,
            self.direction,
        )
    }

    /// Simulates a tick on the person.
    pub fn tick(
        &mut self,
        grid: &Rectangle,
        obstacles: &[Rectangle],
        positions: &[Xy],
        ghosts: &[Xy],
    ) {
        self.rng.tick();

        self.infection_state.in_state_since += 1;

        if self.is_infected() && self.in_state_since() >= self.parameters.incubation_time {
            self.set_state(State::Infectious);
        } else if self.is_infectious() && self.in_state_since() >= self.parameters.recovery_time {
            self.set_state(State::Recovered);
        }

        let acceleration = self.rng.acceleration().vector();
        let velocity = (self.direction.vector() + acceleration).limit(-1, 1);
        let position = self.position + velocity;

        // Check whether we would would bump into a wall.
        if !grid.contains(&position) {
            self.direction = Direction::None;
            return;
        }
        // Check whether we would bump into an obstacle.
        if obstacles.iter().any(|o| o.contains(&position)) {
            self.direction = Direction::None;
            return;
        }
        // Check whether we would bump into another person or their ghost.
        if positions
            .iter()
            .chain(ghosts.iter())
            .any(|p| *p == position)
        {
            self.direction = Direction::None;
            return;
        }

        self.direction = Direction::from_vector(velocity);
        self.position = position;
    }
}

use ring::digest::{digest, SHA256};

/// Random number generator.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Rng {
    parameters: Arc<Parameters>,
    digest: Vec<u8>,
}

impl Rng {
    fn new(seed: &[u8], parameters: Arc<Parameters>) -> Self {
        Self {
            parameters,
            digest: seed.to_vec(),
        }
    }

    fn tick(&mut self) {
        self.digest = digest(&SHA256, &self.digest).as_ref().to_vec();
    }

    fn digest(&self) -> &Vec<u8> {
        &self.digest
    }

    fn unsigned_byte(&self, position: usize) -> usize {
        self.digest[position] as usize
    }

    fn is_coughing(&self) -> bool {
        self.unsigned_byte(0) < self.parameters.cough_threshold
    }

    fn is_breathing(&self) -> bool {
        self.unsigned_byte(1) < self.parameters.breath_threshold
    }

    fn acceleration(&self) -> Direction {
        Direction::from_index(self.unsigned_byte(2) / self.parameters.acceleration_divisor)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use base64::Engine;

    use super::{Parameters, Rng};
    use crate::model::direction::Direction;

    #[test]
    fn test_rng_tick() {
        let initial = base64::engine::general_purpose::STANDARD
            .decode("0pPlYDoCGAumTmfQUlh04ccEXW0+ePysdrb6cDIDsBc=")
            .unwrap();
        let result = base64::engine::general_purpose::STANDARD
            .decode("7cGq16rdQAK1PpRRyosEE4dKCxfNzVzv/Cd+kvONlIk=")
            .unwrap();
        let mut rng = Rng::new(
            initial.as_ref(),
            Arc::new(Parameters::new(1, 1, 1, 1, 1, 1)),
        );
        rng.tick();
        assert_eq!(rng.digest(), &result);
    }

    #[test]
    fn test_rng_special() {
        use base64::Engine;

        let parameters = Arc::new(Parameters::new(30, 150, 20, 120, 7, 8));
        let seed = base64::engine::general_purpose::STANDARD
            .decode("FEa0SttmFeSb+odvm1s6/Bxp+yN/z21W1+JboLch1bk=")
            .unwrap();
        let rng = Rng::new(&seed, parameters);
        assert!(rng.is_coughing());
    }

    #[test]
    fn test_rng() {
        use base64::Engine;

        let parameters = Arc::new(Parameters::new(20, 150, 20, 140, 3, 3));
        let seed = base64::engine::general_purpose::STANDARD
            .decode("XwgjBc/MefpIdtmIAgj4jnFqhqSz1YyE+7UwFEfmj4Y=")
            .unwrap();
        let mut rng = Rng::new(&seed, parameters);

        rng.tick();
        assert_eq!(
            "atRdq1bbo8+I5rbA3bI5dyYO5Rci5SuwbkhwJ+9pBPE=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest())
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "K0XbcKM36gt8RcwZKRE8x3lT7wPWWfA7NCqmKL+PqpU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "l8oZE9RXueChCPwFulJXkjLRe+OvY3obm8GMIPO+JFw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "sPiE0WTI0RwoV/wQm9SDgYUwY3cvBn1WbrOY/a7Lr3I=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::SouthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "rbRk2jPIe9oBHJxW7GxqsKEKbBCbKnSQTXkgOsEGpAM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "QQDGEY2/XuBfNrGu6jyXkCDr9K+6vR6ahdgmUcGSkhI=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "muDNpVXe9TD7udbGWnDTHJmZUqc2nzwlXqJZpddVgec=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "1K9Bqp2DBHGECp6jy4I4Hh+34OcD77TXGbX7fe/ktE4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "rP5Tslc4+bYGDL4TD7/p7Cg67/4jGejhpD6Ct7jm59s=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "TMFX0oAXEH9yh+rvEllJTXTbNjRXf0VK8DG6aCLxrjM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Rd53GOnSkcUhBiJ435ZzZppu6WpFkEWeWTgcLsiAfnE=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "LLTGoAXqpcneO0zIodZi0HpssKqnBEdBsIcFUO4BPM0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "cisRy1dq0uUW2WpSYWHZwkCPdNQ8/bpBO/EMvX4Y46A=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "qjY23Yrr45BGLPmLHkp2kCVZogkGArnLuOZbri1QYvY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "oP0NYoOmtr0IVmJ0ge6svbnsEaF15DjDL6CL/9s67Jk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "N32bisLnYd3S8gOrEPpD1d7U+6oc1b2ya9AukrvDusQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "GTzajIIvue++ADsydjzt6J9iGyk/2bcPTqjnYvJS5SQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "gC5yvZBGwxo8dCy7/0S6oG5g15XNRPBrCM7BqHEe0Cs=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "IzshnmhwYm5KHODvDvLaBWLCXoCRRRGugox5MKA37qE=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fwAMCmrlJY6gbgNwMOqg6/RmmYgRfBQCiCmzMZr2lrA=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "vjL5kTR+Q9c/peViaI715kReyQ6V4aAa0YPY2k0r0Fk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "cHouWKRM+YnbEpv0lZ+kV68X70QU0iSmM6vp3xevgBY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Bobc2ZRW3XiBavCVis545P/cRmlL1IxNM9ABF2gPZjU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fs81OcPk8rfpCi4d9N9GtBmX7ZBb789kfE05PD+ygII=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "eQLnZoemGtoeNSJ9Eu0i0kTfATLBsvs78/BnLjqpU8I=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "tmH3RxzVTIglVPy+kpjaWu3+ac4Cy5wuZCWhZQPBRjE=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "KxWKpIsQF2JFZo37giU8F/mCb/tN+mNyV4NWsmwhEW8=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Wjg0EZr0Z98z4/9wd/Uz9nXtr+tKjFrKXV45utCAxPw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "aaRabu55hYeWoKokJl1/ArUTOn49ril47iIq9Z0ecfc=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "WnlP9BjW/vAWZlvcmC0TqtIcnQQ2QddIefBnnwN4Ljo=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "kX5xAWKDLGiUac/WhbSKzXr9xW1llZjlZOwzBtoFIaM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "41HxhjcP/+PBLUWoKCeOTxu/77gp4BX5HLSscNY7Q9E=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "8ZzN3/BXMCYE72liBbghBv8IYZJ+b8S571qBJ9KVNvg=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "DfKzC/9ouxQa4qLYtqBNoflVyX9s/0fJwFfO4Vb7Hk4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "COsepNgUTdJMcAENizazMsE9PiWRELpvpPDRHyxJNwU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "SE9k3MbgS6pGxAH5uO//o1WtGlP0lCju+2eNHR4im5M=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fLY9V79Krb5O+WqQXotjBs1XJySl0ZZm5bjVTUOvL68=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "te5WBsaxtHCbOYw6/wTCKO/88HVT1kb0dqWYerRXaxc=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "DY00MlMjZvBDh1Zzze5cXeL4sd/CIfZ8aocFa4zB4JQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "u8v/ursa9Z8FqA62fuqv/S5CIweOkAYFgJ10ryr37Xc=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "m2ISBHxK8G28EDITmnMtTeHa0r9e8vwfu4FVi4AMoK4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "g1ZNWWquAdUsmRXALX+rIJ/5eHQdEuy1/W9tiqExm8w=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "o86wCkAGz4SRDl7oyt912MtQEXOkYlppslzFLCIZtEw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "BBvEug/uaxUmCYfWndiKoMV7Fv4+drvWdvCtFMwKn7Y=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "yL8k7t/3XVFb3/xxzIXnl7Yejgf5td9ILT7GjDtopdw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "MLvSrcMd9CwL8dVqMKrgcpsjCk/TAx6jxE5QnWo+va0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "0PAk2pQw/nqp4wxzrqdlol5DDzK2WKDTeQW0edg3b10=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "ojc35EgDq3koy/0D38rgr3+txRzYMiKPYfNf/zr1QRM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "AyHaMTjGCEL84t+1zG7KmqKDu9O55UDfcbF1vMk5CvY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "H2A0YCGc0i0VSMWdu6up18qR/elpw32a5/DoUN3TGAs=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "CR9pB2j4deIJDQ5+rodhN/Y0vk+nCef9FLlaeKC8Fxk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "5GB1wNTUfFeFaqYA6BtEsTWqyNoSnip4xrY5rGrvp0k=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "uws64toWUQCSoVUegrePefHnDLRHZ3NxNdDOysVIWZ4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fJuuLocyXxttFWDIt2MV4Khw/7fhExc6ek4tAWEcXBk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "yTmZpirG1MDYnzmilF5YUdB38MsKQ64Wu1Ziwvmd3yU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "sygAVPPa1GA/tttfYF7NGqVmgvKgQQxh4VUvee5s3wI=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "XM/uhA6KnkUSf3ZbgBVlBRnTu7mGibUqNtzOUi3ultY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "C9brLkIHaf0iUwOiWp8px9Crzj7yyB6jJOIUCUKQwgQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "k9gxibxMxV0LhQ61iJA5qR/nV9KjBSz5MfxcNmlpFYQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "/i1PS+WDVPfdbXTDte7jfZ8W3zd7MmE5Iq7lq0S8L1U=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "TPEAUZtl9WJwfSkjZaEolzcobHMt4DW/30MtRloSyww=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "ix647xKC7zTRCVzwbovBTP4nFpx1ZotGaUFk5nlMnB8=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "SqOntHV9ix177bXQ9oqYdPSUny/3UbX5UIrHgtyjrFk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "pu0mHEAAUuaik0osLOKd2SvrL9KV9HbucuX0KYjTQjo=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "3oJm2fq63jh9jzDwVD7gXk6yQmTUKrMeM0yOLfL+kXQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "HGN1mnAdhrX8iQLcE1/aCuMcyV90Y2p8SyqOkb4DAUI=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "qhBKliFqKAlFQMBm6i7h7+r6hvgrpsrI3JAYLj2w7dM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::West, rng.acceleration());

        rng.tick();
        assert_eq!(
            "5ysO9Ng4qXye2d6f8oeigJNDD7DUhpVSKLDpmmVWuPQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::North, rng.acceleration());

        rng.tick();
        assert_eq!(
            "FQgwWlPyM7Jk4wfhMzYrAJDNGGQs2e7tNY3Rqru1mbU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "f5vERek/Sm8rc8b+HFomYCErG9XB7/dRJCDPUkoy2XQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "tG+VjdOpqavqrJpglQl3wMvwxBkiDPx/6gAqmcYDEu0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "9vUmaOlSfWmmiLE7ghLTjn84+0lSK1K8p/WRx/UXTq0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "u6K7BfoVXylpsTxNTO7XLYq2IAdhAFZYO6gt9OCHeDs=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "sj8U9eQzUkdTFKA8qwCHzyfLrmjA4V4FAKf3mIko9t4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "0RGorIYrK3UrsADNtSDrDO+vRnn1Rm7XqaAS7fGgDSc=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "E2YUGJnxvW2b5S20UvRztMSJAG1wMV+wbls1MIlMQEU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "slRoJQzr4BE5ZptzjA5oLWT1y9zkGVZoYLIRLscYdCA=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "VnDAyiEe+xS4qoO3zxBAQYAugsxkXV649gE043uQTvY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Pe/00i28BIbQy00oX23VZLTmHNlbdAWTkuole3qIM9g=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "jiykR+Pl6jEHOZuo1u/VXL0wtHwT0+0jtNZV1S1kO+c=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "My13yxIsKokPR5Z5jqd1S82i43GkHqckZnJE7sauvNU=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fUm7qu6BNLLHztWI6Xjv7/ijtCZCOFvfqdkWm5hnlbE=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "vBLx3IZjiNHx6qMstwgHfN18K9MDmu7ZjQmT4DbAUq8=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "WpGV9Wutn5N98kGj3iz+8yK25BxckFFWfbjZjJKhN4Y=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "1gt2e/lCHlWOaDs4bHipQRYfebCoMGFvjOOWz7U6AzA=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Upyp7xZ8Eerrd+IEcIYfQLB6sbz/QJUaTR57Mtub/2I=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "sqkawfb+VO+8oOdhuRtMdLT5d8WzDg61NRfak2QX4p0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "7xzCrNSiGzS92zv0TvonDwLH/xWjY65GyJU2HxbrBMw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "3uOpNbK5S+GLoTcAVSx6rvVHmxsuiSyv048PFYr8aIw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "+bFQUJ6Dbp834WDa401lNzF1lt+rkjwKztLiwVnGHVQ=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "Wr/7cOoxW1CDU/tt7g3M06FR4YneDglB8HfcdzQ1dkk=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "p7IUdOEHBf1NPkWvaLK4sS0FcexjoVfs4C3guNG9PrM=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "5Oclo0t7scM5Xajl62Snf1HLC0irIG/p/kLlm5x9Fi4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::East, rng.acceleration());

        rng.tick();
        assert_eq!(
            "fg15jsfK9lVnMCMA53x+qUUvFQfNMLFmtPFP/KypRA0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "p9KrJMtMwnFlOA1pqcVXMF8QHdQxRiWyrxJWHaE6CU4=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::None, rng.acceleration());

        rng.tick();
        assert_eq!(
            "gkmHLZq14rbSkWfLgobukcV0s26IxaaB8ZYO6A6QF30=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthEast, rng.acceleration());

        rng.tick();
        assert_eq!(
            "v6dzS148+d4BfBA9Nm5uyQ06WGcKV7FekaVZ/GwE0Iw=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(false, rng.is_breathing());
        assert_eq!(Direction::NorthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "N1spijkzA8nMTEbKubkp45jvquBJ8MnPuR+WCLA32E0=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());

        rng.tick();
        assert_eq!(
            "E0eMwTxYfnPC12hf7iq3o7sUtkv4YdlVNWmBEvWhwWY=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(true, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::SouthWest, rng.acceleration());

        rng.tick();
        assert_eq!(
            "NQg017ydJRyymRSzwL+fPkCEDOrkg9k6EeLMCEkLQxA=",
            base64::engine::general_purpose::STANDARD.encode(rng.digest()),
        );

        assert_eq!(false, rng.is_coughing());
        assert_eq!(true, rng.is_breathing());
        assert_eq!(Direction::South, rng.acceleration());
    }
}
