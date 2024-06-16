use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use spread_sim_core::{
    model::{
        partition::Partition, rectangle::Rectangle, scenario::Scenario, statistics::Statistics,
        xy::Xy,
    },
    simulation::{Person, PersonId},
    validator::Validator,
};

use crate::util::{OutputMod, TraceEntryWithId};

pub struct Patch {
    scenario: Scenario,
    patch_id: usize,

    validator: Arc<dyn Validator>,
    population: Vec<Person>,
    ticks_total: usize,
    ticks_independent: usize,
    positions: HashMap<PersonId, Xy>,
    ghosts: Vec<Xy>,
    padded_patch: Rectangle,
    owned_patch: Rectangle,
    obstacles: Vec<Rectangle>,

    trace: Vec<TraceEntryWithId>,
    statistics: HashMap<String, Vec<Statistics>>,
    // every patch has a sender channel for each neighboring patch which sends a vector of people
    // to the neighbor patch
    send_channels: Vec<Sender<Vec<Person>>>,
    // every sender from a neighboring patch will have a corresponding reciever channel in the
    // patch itself to recieve the sent values from the neighbors
    rec_channel: Vec<Receiver<Vec<Person>>>,
    //a channel to send (return) the final output of the patches to the main program
    return_channel: Sender<OutputMod>,
}

//   0    ___1___    2
//       |       |
//   3   |       |   4
//       |_______|
//   5       6       7

impl Patch {
    pub fn new(
        scenario: &Scenario,
        population: &Vec<Person>,
        patch_id: usize,
        validator: Arc<dyn Validator>,
        ticks_independent: usize,
        padding: usize,
        send_channels: Vec<Sender<Vec<Person>>>,
        rec_channel: Vec<Receiver<Vec<Person>>>,
        return_channel: Sender<OutputMod>,
    ) -> Self {
        let mut part_vec = scenario.partition.clone();
        part_vec.x.insert(0, 0);
        part_vec.y.insert(0, 0);
        part_vec.x.push(scenario.grid_size.x);
        part_vec.y.push(scenario.grid_size.y);
        let (padded_patch, owned): (Rectangle, Rectangle) =
            create_padded_patch(patch_id, &part_vec, padding);
        let obstacles: Vec<Rectangle> = filter_obstacles(scenario.obstacles.clone(), &padded_patch); //returns all obstancles in our scenario that are icluded in our patch area
        let pops: Vec<Person> = filter_persons(population.clone(), &padded_patch); //returns all people in our scenario that are icluded in our patch area
        let statistics = scenario
            .queries
            .keys()
            .map(|key| (key.clone(), Vec::new()))
            .collect();
        let positions = pops.iter().map(|p| (p.id, p.position)).collect(); // obtain the position of any person quickly by storing them with their id as index
        let mut out = Patch {
            ticks_total: scenario.ticks,
            scenario: scenario.clone(),
            patch_id,
            validator,
            ghosts: Vec::with_capacity(pops.len()), /* only a person that moves can lead to
                                                     * creation of a ghost,so we have maximum
                                                     * pops.len() ghosts */
            population: pops,
            ticks_independent,
            positions,
            padded_patch,
            owned_patch: owned,
            obstacles,
            trace: Vec::new(),
            statistics,
            send_channels: send_channels,
            rec_channel: rec_channel,
            return_channel,
        };
        out.extend_output();
        out
    }

    pub fn simulate(&mut self) {
        let mut tick = 0;
        //run simulation until total number of ticks
        while tick < self.ticks_total {
            self.validator.as_ref().on_patch_tick(tick, self.patch_id);

            //check everytime we fulfill the number of independent ticks we have so that we
            // synchronise after every cycle
            if (tick + 1) % self.ticks_independent == 0 {
                self.wipe_padding(); //remove people on the padding because they have error values
                self.sync(); //sync after removing padding to add the correct values for
                             // people into the paddings
            }
            self.tick(tick); //simulate a tick
            tick += 1; //increase the nb of ticks
        }

        self.wipe_padding();
        self.finish();
    }

    fn tick(&mut self, tick: usize) {
        //simulate a tick over every person in the patch
        for person in self.population.iter_mut() {
            self.ghosts.push(person.position); //the position of any person becomes a ghost after he moves QUESTION : when he stays in
                                               // place,does the place also become a ghost?
            self.validator
                .as_ref()
                .on_person_tick(tick, self.patch_id, person.id);
            person.tick(
                &self.padded_patch,
                &self.obstacles,
                &self.positions.iter().map(|p| *p.1).collect::<Vec<Xy>>(),
                &self.ghosts,
            ); //simulate a tick on a person

            //according to the invariant, index exists
            *self.positions.get_mut(&person.id).unwrap() = person.position; //update the
                                                                            // new positions
        }

        // Bust all ghosts.
        self.ghosts.clear();

        //Here is where magic happens,we check whether there is some change of states based on
        // the new positions and the conditions surrounding them by having a nested for loop to
        // compare all people with each other
        for i in 0..self.population.len() {
            for j in i + 1..self.population.len() {
                let pos_i = self.population[i].position;
                let pos_j = self.population[j].position;

                let delta_x = (pos_i.x - pos_j.x).abs();
                let delta_y = (pos_i.y - pos_j.y).abs();

                let distance = (delta_x + delta_y) as usize; //calculates a distance between 2 people
                if distance <= self.scenario.parameters.infection_radius {
                    //In this case,infection is possible if necessary cpondituons are met.
                    //a person should be infectious and coughing to infect the other person
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

    fn sync(&mut self) {
        for channel in self.send_channels.as_slice() {
            channel.send(self.population.clone()).unwrap(); //send the population through
                                                            // every channel
        }

        self.positions.clear(); //remove positions

        for channel in self.rec_channel.as_slice() {
            let new_ppl: Vec<Person> = channel.recv().unwrap(); //store people recieved
            let mut filtered_ppl = filter_persons(new_ppl, &self.padded_patch); //obtain only the people that are in the padding,since this is the area where the error
                                                                                // happens so this is where we should update the population through our recieved values
            self.population.append(&mut filtered_ppl); //add the recieved people that
                                                       // belong in the padded patch to
                                                       // the population
        }
        self.population
            .sort_by(|a, b| usize::from(a.id).cmp(&usize::from(b.id))); //sort the new population based on id since ticks depend on the order of the id of people
        for p in &self.population {
            self.positions.insert(p.id, p.position); //re-add the positions of the
                                                     // population
        }
    }

    //clear the padding area from people so that it could be refilled by the new error-free
    // values
    fn wipe_padding(&mut self) {
        let tmp = filter_persons(self.population.clone(), &self.owned_patch); //keep the people found in original patch
        self.population.clear();
        self.population = tmp; //re-initialise the population to only contain the people
                               // in the original patch
    }

    fn count_persons(&self, pred: impl Fn(&Person) -> bool) -> u64 {
        return self.population.iter().filter(|person| pred(person)).count() as u64;
    }

    fn extend_output(&mut self) {
        if self.scenario.trace {
            self.trace.push(TraceEntryWithId::new(
                self.population
                    .iter()
                    .filter(|p| self.owned_patch.contains(&p.position))
                    .map(|p| (p.info(), p.id))
                    .collect(),
            ));
        }
        self.extend_statistics();
    }

    fn extend_statistics(&mut self) {
        for (key, query) in &self.scenario.queries {
            let statistics = Statistics::new(
                self.count_persons(|p| {
                    p.is_susceptible()
                        && query.area.contains(&p.position)
                        && self.owned_patch.contains(&p.position)
                }),
                self.count_persons(|p| {
                    p.is_infected()
                        && query.area.contains(&p.position)
                        && self.owned_patch.contains(&p.position)
                }),
                self.count_persons(|p| {
                    p.is_infectious()
                        && query.area.contains(&p.position)
                        && self.owned_patch.contains(&p.position)
                }),
                self.count_persons(|p| {
                    p.is_recovered()
                        && query.area.contains(&p.position)
                        && self.owned_patch.contains(&p.position)
                }),
            );
            // According to the type's invariants, the entry for the key exists.
            self.statistics.get_mut(key).unwrap().push(statistics);
        }
    }

    fn finish(&mut self) {
        self.return_channel
            .send(OutputMod::new(self.trace.clone(), self.statistics.clone()))
            .unwrap();
    }
}

//checks for people inside a certain area(rectangle) by checking their position and
// returns all people inside this area
fn filter_persons(ppl: Vec<Person>, acceptable_area: &Rectangle) -> Vec<Person> {
    let mut inside = Vec::new();
    for p in ppl {
        if acceptable_area.contains(&p.position) {
            inside.push(p);
        }
    }
    inside
}

//retrun obstacles that are inside a certain patch,which can block a spread of infection
// into another patch
fn filter_obstacles(obs: Vec<Rectangle>, acceptable_area: &Rectangle) -> Vec<Rectangle> {
    let mut overlaps = Vec::new();
    for o in obs {
        if acceptable_area.overlaps(&o) {
            overlaps.push(o);
        }
    }
    overlaps
}

//returns padded patch & vector of !!disjoint!! paddings
//
//
/// The whole area is returned as [`Rectangle`]
/// Arg [`Partition`] needs to contain origin as the first entry and lower right corner of
/// the grid rectangle as the last entry
///  The center area is "owned"
///
/// Returns (PaddedPatch, OwnedPatch)

pub fn create_padded_patch(
    patch_id: usize,
    splits: &Partition,
    padding: usize,
) -> (Rectangle, Rectangle) {
    let columns = splits.x.len() - 1; //number of vertical splits
    let rows = splits.y.len() - 1; //number of horizontal splits

    if patch_id >= columns * rows {
        panic!("patch_id out of range");
    }

    let is_top_row = patch_id < columns; // first row has patch ids till columns,while 2nd row has ids from (columns+1) till
                                         // (columns*2)...
    let is_left_edge = patch_id % columns == 0;
    let is_right_edge = (patch_id + 1) % columns == 0;
    let is_bottom_row = patch_id >= columns * (rows - 1);

    let mut patch_origin = Xy::new(splits.x[patch_id % columns], splits.y[patch_id / columns]); //gets the left upper_corner point (origin) of the rectangle representing the patch

    let mut patch_size = Xy::new(
        splits.x[patch_id % columns + 1],
        splits.y[patch_id / columns + 1],
    ) - patch_origin; //calculates the distance of the diagonal of upper_corner and lower_corner

    let owned = Rectangle::new(patch_origin, patch_size); //creates the rectangle corresponding

    //println!("{}, {}", patch_size, patch_origin);
    // println!(
    //     "Edge bools: top {}; left {}; right {}; bottom {}",
    //     is_top_row, is_left_edge, is_right_edge, is_bottom_row
    // );

    let min_coords = Xy::new(splits.x[0], splits.y[0]); //gets the top left corner coordinates of the scenario
    let max_coords = Xy::new(
        splits.x[0] + splits.x[columns],
        splits.y[0] + splits.y[rows],
    ); //gets the bottom rightt corner coordinates of the scenario

    //check for every edge case aka everytime a patch is located at some border and add the
    // padding accordingly

    if !is_bottom_row {
        let bottom_space = max_coords.y - (patch_origin.y + patch_size.y); //gets all the space under our patch

        //if the padding is greater than the space then we should only add bottom_space to the
        // patch size because  if we add padding it will overflow
        if bottom_space < padding as isize {
            patch_size = patch_size + (0, bottom_space);
        } else {
            patch_size = patch_size + (0, padding as isize);
        }
    }

    //similair logic is applied to the other cases

    if !is_top_row {
        let top_space = patch_origin.y - min_coords.y;
        if top_space < padding as isize {
            patch_origin = patch_origin - (0, top_space);
            patch_size = patch_size + (0, top_space);
        } else {
            patch_origin = patch_origin - (0, padding as isize);
            patch_size = patch_size + (0, padding as isize);
        }
    }
    if !is_right_edge {
        let right_space = max_coords.x - (patch_origin.x + patch_size.x);
        if right_space < padding as isize {
            patch_size = patch_size + (right_space, 0);
        } else {
            patch_size = patch_size + (padding as isize, 0);
        }
    }
    if !is_left_edge {
        let left_space = patch_origin.x - min_coords.x;
        if left_space < padding as isize {
            patch_origin = patch_origin - (left_space, 0);
            patch_size = patch_size + (left_space, 0);
        } else {
            patch_origin = patch_origin - (padding as isize, 0);
            patch_size = patch_size + (padding as isize, 0);
        }
    }

    (Rectangle::new(patch_origin, patch_size), owned)
}

#[cfg(test)]
mod test {
    use spread_sim_core::model::{partition::Partition, rectangle::Rectangle, xy::Xy};

    use super::create_padded_patch;

    #[test]
    fn test_single_patch() {
        let grid = Xy::new(10, 10);
        let (pat, rects) =
            create_padded_patch(0, &Partition::new(vec![0, grid.x], vec![0, grid.y]), 6);
        assert_eq!(rects, Rectangle::new(Xy::new(0, 0), grid));
        assert_eq!(pat, Rectangle::new(Xy::new(0, 0), grid));
    }

    #[test]
    fn test_single_x_split() {
        let grid = Xy::new(10, 5);
        let (pat, rects) =
            create_padded_patch(0, &Partition::new(vec![0, 5, grid.x], vec![0, grid.y]), 4);
        assert_eq!(rects, Rectangle::new(Xy::new(0, 0), Xy::new(5, 5)));
        assert_eq!(pat, Rectangle::new(Xy::new(0, 0), Xy::new(5 + 4, 5)));
    }

    #[test]
    fn test_single_x_split_right() {
        let grid = Xy::new(10, 5);
        let (pat, rects) =
            create_padded_patch(1, &Partition::new(vec![0, 5, grid.x], vec![0, grid.y]), 4);
        assert_eq!(rects, Rectangle::new(Xy::new(5, 0), Xy::new(5, 5)));
        assert_eq!(pat, Rectangle::new(Xy::new(1, 0), Xy::new(5 + 4, 5)));
    }

    #[test]
    fn test_single_y_split() {
        let grid = Xy::new(5, 10);
        let padding = 4;
        let (pat, rects) = create_padded_patch(
            0,
            &Partition::new(vec![0, grid.x], vec![0, 5, grid.y]),
            padding,
        );
        assert_eq!(rects, Rectangle::new(Xy::new(0, 0), Xy::new(5, 5)));
        assert_eq!(
            pat,
            Rectangle::new(Xy::new(0, 0), Xy::new(5, (5 + padding) as isize))
        );
    }

    #[test]
    fn test_single_y_split_bottom() {
        let grid = Xy::new(5, 10);
        let padding = 4;
        let (pat, rects) = create_padded_patch(
            1,
            &Partition::new(vec![0, grid.x], vec![0, 5, grid.y]),
            padding,
        );
        assert_eq!(rects, Rectangle::new(Xy::new(0, 5), Xy::new(5, 5)));
        assert_eq!(
            pat,
            Rectangle::new(Xy::new(0, 1), Xy::new(5, (5 + padding) as isize))
        );
    }

    #[test]
    fn test_split_complex_central() {
        let grid = Xy::new(15, 15);
        let padding: isize = 3;
        let (pat, rects) = create_padded_patch(
            4,
            &Partition::new(vec![0, 5, 10, grid.x], vec![0, 5, 10, grid.y]),
            padding as usize,
        );
        assert_eq!(
            pat,
            Rectangle::new(
                Xy::new(5 - padding, 5 - padding),
                Xy::new(5 + 2 * padding, 5 + 2 * padding)
            )
        );
        assert_eq!(rects, Rectangle::new(Xy::new(5, 5), Xy::new(5, 5)));
    }
}
