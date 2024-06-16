use std::collections::HashSet;

use crate::model::{rectangle::Rectangle, scenario::Scenario, xy::Xy};

/// Computes whether it is possible to propagate information from a *source area*
/// to a *target area* after an arbitrary amount of ticks.
///
/// You may use this method to check whether it is possible to propagate information
/// from the padding of a patch inside the area owned by the patch. If you do not want
/// to use this method make sure your method is as least as precise as this method.
///
/// For those who would like to earn a bonus: In some cases this method returns that
/// information may propagate although on closer inspection this is not the case. What
/// are those cases? Can you improve on that?
pub fn may_propagate_from(scenario: &Scenario, source: &Rectangle, target: &Rectangle) -> bool {
    let mut frontier: Vec<Xy> = Vec::new();
    let mut region: HashSet<Xy> = HashSet::new();
    for target_cell in target {
        if !scenario.on_obstacle(&target_cell) {
            frontier.push(target_cell);
            region.insert(target_cell);
        }
    }

    let infection_radius = scenario.parameters.infection_radius as isize;

    while let Some(cell) = frontier.pop() {
        for delta_x in -infection_radius..infection_radius + 1 {
            for delta_y in -infection_radius..infection_radius + 1 {
                let distance = (delta_x + delta_y).abs();
                if distance <= infection_radius || (delta_x.abs() <= 1 && delta_y.abs() <= 1) {
                    let neighbor = cell + Xy::new(delta_x, delta_y);
                    if !region.contains(&neighbor)
                        && scenario.grid().contains(&neighbor)
                        && !scenario.on_obstacle(&neighbor)
                    {
                        frontier.push(neighbor);
                        region.insert(neighbor);
                    }
                }
            }
        }
    }

    for source_cell in source {
        if scenario.on_obstacle(&source_cell) {
            continue;
        }
        if region.contains(&source_cell) {
            return true;
        }
    }

    false
}
