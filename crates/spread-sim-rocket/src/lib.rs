use std::{
    collections::VecDeque,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::spawn,
};

use spread_sim_core::{
    model::{output::Output, scenario::Scenario, trace::TraceEntry},
    simulation::{may_propagate_from, Person},
    validator::Validator,
    InsufficientPaddingError,
};
use util::OutputMod;

use crate::patch::{create_padded_patch, Patch};

mod patch;
mod util;

static DEBUG: bool = false;
/// Launches your concurrent implementation. 🚀
///
/// You must not modify the signature of this function as our tests rely on it.
///
/// Note that the [`Validator`] is wrapped in an [`Arc`] and can be cloned and passed
/// around freely (it is [`Sync`] and [`Send`]).
///
/// - *scenario*: The [`Scenario`] to simulate.
/// - *padding*: The padding to use for the simulation.
/// - *validator*: The [`Validator`] to call (for testing).
/// - *starship*: Indicates whether the implementation of assignment 2 should be used.
pub fn launch(
    scenario: Scenario,
    padding: usize,
    validator: Arc<dyn Validator>,
    starship: bool,
) -> Result<Output, InsufficientPaddingError> {
    // The next line is here simply to suppress unused variable warning. You should
    // remove it and actually use the provided arguments. ;)
    if starship {
        // Launch your starship here. (assignment 2)
        //
        // Note that you may ignore the padding and validator parameters in this case.
        //
        // Please keep the panic in case you are not implementing assignment 2. Our test
        // infrastructure uses it to determine whether you implemented assignment 2.
        panic!("Starship has not been implemented.")
    } else {
        //+1 allows min of 1 tick
        if padding < scenario.parameters.infection_radius + 2 {
            return Err(InsufficientPaddingError::new(padding));
        }

        let pop: Vec<Person> = scenario
            .population
            .iter()
            .enumerate()
            .map(|(id, info)| Person::new(id.into(), info, scenario.parameters.clone()))
            .collect(); //create a Person collection from the population

        let (out_ret_sender, ret_chan) = channel(); //return channel
                                                    //the number of patches is determined by how many splits there are
        let patches = (scenario.partition.x.len() + 1) * (scenario.partition.y.len() + 1);

        //create index invariant for all building blocks
        let mut areas = Vec::with_capacity(patches); //create a rectangle to represent all patches and includes only the owned patches(without
                                                     // the padding)
        let mut padded_areas = Vec::with_capacity(patches);

        if DEBUG {
            println!(
                "Splits in X: {:?}, Splits in Y: {:?}",
                scenario.partition.x, scenario.partition.y
            );
        }

        let mut partition_arg = scenario.partition.clone(); //create a clone of Partition but including the outlines aka the starting and ending
                                                            // borderlines
        partition_arg.x.insert(0, 0);
        partition_arg.y.insert(0, 0);
        partition_arg.x.push(scenario.grid_size.x);
        partition_arg.y.push(scenario.grid_size.y);
        for i in 0..patches {
            let tmp = create_padded_patch(i, &partition_arg, padding); //returm (owned_patch,padded_patch)
            areas.push(tmp.1); //add the original patch to areas
            padded_areas.push(tmp.0); //add the padded patch to padded_areas
        }

        let mut vec_of_senders: VecDeque<Vec<Sender<_>>> = VecDeque::with_capacity(patches); //stores all the sender channels of each patch
        let mut vec_of_receivers: VecDeque<Vec<Receiver<_>>> = VecDeque::with_capacity(patches); //stores all the reciever channels of each patch
        for _ in 0..patches {
            vec_of_senders.push_back(Vec::new());
            vec_of_receivers.push_back(Vec::new());
        }

        let ind_ticks = calc_independent_ticks(
            padding,
            scenario.parameters.incubation_time,
            scenario.parameters.infection_radius,
        ); //use calc_independent_ticks to calulate how many ticks a patch can do in each cycle

        //loop over each patch and check whether one's padding overlaps another,in case they do
        // we establish a communication channel between them
        for i in 0..patches {
            for j in (i + 1)..patches {
                if padded_areas[i].overlaps(&areas[j]) {
                    let overlap = padded_areas[i].intersect(&areas[j]);
                    //check whether obstacles block possible communication between intersecting
                    // patches.
                    if may_propagate_from(&scenario, &overlap, &areas[i]) {
                        //disease can spread between patch i and j here
                        let (from_i, to_j) = channel(); //channel to send from i to j
                        let (from_j, to_i) = channel(); //channel to send from j to i

                        vec_of_senders[i].push(from_i); //add another sender to patch i
                        vec_of_receivers[i].push(to_i); //add another reciever to patch i
                        vec_of_senders[j].push(from_j);
                        vec_of_receivers[j].push(to_j);
                    }
                }
            }
        }

        let scenario_clone = scenario.clone();
        for i in 0..patches {
            let senders = vec_of_senders.pop_front().unwrap(); //assign senders[i] to patch[i]
            let receivers = vec_of_receivers.pop_front().unwrap(); //assign recievers[i] to patch[i]
            let tmp_validator = validator.clone();
            let ret_sender = out_ret_sender.clone();
            let c = pop.clone(); //NO
            let b = scenario.clone();

            spawn(move || {
                Patch::new(
                    &b,
                    &c,
                    i,
                    tmp_validator,
                    ind_ticks,
                    padding,
                    senders,
                    receivers,
                    ret_sender,
                )
                .simulate()
            }); //runs a thread on a patch
        }
        drop(out_ret_sender);

        let mut out: OutputMod = ret_chan.recv().unwrap();
        //iterate over every patch to recieve their values once they finish one at a time
        for _ in 1..patches {
            let mut new_data = ret_chan.recv().unwrap();

            for i in 0..out.trace.len() {
                out.trace[i]
                    .population
                    .append(&mut new_data.trace[i].population);
            }

            for (a, b) in new_data.statistics {
                if b.len() > 0 {
                    let x = out.statistics.get_mut(&a).unwrap();
                    for i in 0..x.len() {
                        x[i].add(&b[i]);
                    }
                }
            }
        }
        //sort the output into the proper format
        let mut traces = Vec::with_capacity(out.trace.len());
        for i in 0..out.trace.len() {
            out.trace[i].population.sort_by(|a, b| a.1.cmp(&b.1));
            traces.push(TraceEntry::new(
                out.trace[i]
                    .population
                    .iter()
                    .map(|p| p.0.clone())
                    .collect(),
            ));
        }
        let real_out = Output::new(scenario_clone, traces, out.statistics);

        return Ok(real_out);
    }
}

//ERROR:
//tick 1: +infec radius +2
//tick 2: +2
//...
//tick incTime+1: +infec radius +2
fn calc_independent_ticks(
    padding: usize,
    incubation_time: usize,
    infection_radius: usize,
) -> usize {
    //After 1st tick,the error increased by infection_radius + (Inwardmove and outwards move
    // (explained in detail above))
    let mut remaining = padding - 2 - infection_radius; //represent how much of the padding has been overtaken by the infection (the error)
    let mut ticks = 1; //first tick

    //iterate until there's no more remaining,or remaining becomes less than infection_radius
    while remaining > 0 {
        //in case we're in incubation_time,we only increase error by 2
        for _ in 1..incubation_time {
            if remaining <= 1 {
                break;
            }
            remaining -= 2; //increase of error by 2
            ticks += 1; //tick successfully occurs
        }
        if remaining <= infection_radius + 1 {
            break; //remaining became too small in comparison to indection radius,so we
                   // need to stop and synchronise
        }
        ticks += 1; //tick successfully occurs
        remaining -= infection_radius + 2; //crossed incubation time,we could have a new
                                           // infectious person now,so error could
                                           // additionally increase by new infectious
                                           // radius
    }

    ticks
}

#[cfg(test)]
mod test {
    use crate::calc_independent_ticks;

    #[test]
    fn ind_ticks_7() {
        let ticks = calc_independent_ticks(7, 3, 5);
        assert_eq!(ticks, 1);
    }

    #[test]
    fn ind_ticks_10() {
        let ticks = calc_independent_ticks(10, 3, 5);
        assert_eq!(ticks, 2);
    }

    #[test]
    fn ind_ticks_15() {
        let ticks = calc_independent_ticks(15, 3, 5);
        assert_eq!(ticks, 3);
    }

    #[test]
    fn ind_ticks_16() {
        let ticks = calc_independent_ticks(28, 2, 6);
        println!("{}", ticks);
        assert_eq!(ticks, 5);
    }
}
