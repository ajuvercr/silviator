#![feature(fn_traits)]
#![feature(unboxed_closures)]
use std::{
    collections::BinaryHeap,
    error::Error,
    fs::File,
    io::stdin,
    time::{Duration, Instant},
};

use crate::{graphs::try_oo, models::*};

use std::io::BufRead;
mod graphs;
mod models;

#[allow(unused)]
fn simple_turn(state: &mut State) -> Option<()> {
    let current_planets: Vec<_> = state.planets().iter().map(|p| p[0]).collect();

    let friendly: Vec<_> = current_planets.iter().filter(|x| x.owner == ME).collect();
    let enemy: Vec<_> = current_planets.iter().filter(|x| x.owner != ME).collect();

    let source = friendly.iter().max_by(|x, y| x.ships.cmp(&y.ships))?;

    let target = enemy.iter().min_by(|x, y| x.ships.cmp(&y.ships))?;

    state.add_turn(source.id, target.id, source.ships - 1);

    Some(())
}

#[derive(Debug)]
pub struct UsablePlanet {
    pub id: usize,
    pub dist: usize,
    pub usable_ships: i32,
}

#[derive(Debug)]
pub struct OptionalOperation {
    weight: f32,
    pub duration: usize,
    pub required_ships: i32,
    // Self id, duration, and usable ships at that moment
    pub usable_planets: Vec<UsablePlanet>,
    pub target: usize,
}

impl OptionalOperation {
    pub fn score(&self) -> f32 {
        (self.required_ships as f32 * 3.0 + self.duration as f32 * 2.) * self.weight
    }
}

impl PartialEq for OptionalOperation {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.duration == other.duration
            && self.weight == other.weight
    }
}

impl Eq for OptionalOperation {}

impl PartialOrd for OptionalOperation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OptionalOperation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let this_score = self.score();
        let other_score = other.score();

        this_score.partial_cmp(&other_score).unwrap().reverse()
    }
}

fn find_optional_operations(
    target: &PlanetStates,
    states: &State,
    queue: &mut BinaryHeap<OptionalOperation>,
) {
    // TODO this can just be an array
    let mut distances: Vec<Vec<(&PlanetStates, usize)>> = Vec::new();

    let mut maybe_insert = |p, d| {
        while distances.len() <= d {
            distances.push(Vec::new());
        }
        distances[d].push(p);
    };

    for p in states.planets() {
        let d = p.distance(target);
        maybe_insert((p, d), d);
    }

    let mut closers = Vec::new();
    for (d, options) in distances.into_iter().enumerate() {
        closers.extend(options);

        if target[d].owner == ME {
            continue;
        }

        let required_ships = target[d].ships + 1;
        let mut usable_planets = Vec::new();

        for (o, actual_dist) in &closers {
            let actual_required = o[d - actual_dist];
            if actual_required.owner != ME {
                continue;
            }

            usable_planets.push(UsablePlanet {
                id: o.id(),
                dist: *actual_dist,
                usable_ships: actual_required.ships,
            });
        }

        if usable_planets.iter().map(|x| x.usable_ships).sum::<i32>() > required_ships {
            let oo = OptionalOperation {
                weight: 1.,
                duration: d,
                required_ships,
                usable_planets,
                target: target.id(),
            };

            queue.push(oo);
        }
    }
}

fn best_planet(state: &mut State, started: Instant) -> Option<()> {
    let mut b_heap = BinaryHeap::new();
    state
        .planets()
        .iter()
        .for_each(|p| find_optional_operations(p, state, &mut b_heap));

    if let Some(turn) = b_heap.pop() {
        eprintln!("Found turn {:?}", turn);
        let mut req = turn.required_ships + 1;
        let target = turn.target;

        let total_usable: i32 = turn.usable_planets.iter().map(|u| u.usable_ships).sum();

        for UsablePlanet {
            id,
            dist,
            usable_ships,
        } in &turn.usable_planets
        {
            if *dist == turn.duration {
                let sending = usable_ships * req / total_usable;
                state.add_turn(*id, target, sending);
                req -= sending;
            }
        }

        let mut tried = vec![turn];
        let mut max_len = 0;
        while started.elapsed() < Duration::from_millis(800) && !b_heap.is_empty() {
            if let Some(turn) = b_heap.pop() {
                tried.push(turn);
                let o = try_oo(&tried, state);
                max_len = max_len.max(o.len());
            } else {
                break;
            }
        }
        eprintln!("found {}: {}", tried.len(), max_len);
    }

    Some(())
}

fn turn(state: &mut State, started: Instant) {
    best_planet(state, started);

    let output = state.flush();
    println!("{}", output);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut lines = stdin().lock().lines();

    let line = lines.next().unwrap()?;
    let now = Instant::now();
    let input = serde_json::from_str::<Input>(&line).unwrap();
    let mut state = State::new(input);

    let mut turn_count = 0;

    eprintln!("-------------------------  Turn {}", turn_count);
    turn(&mut state, now);
    turn_count += 1;

    while let Some(Ok(line)) = lines.next() {
        let now = Instant::now();
        let input = serde_json::from_str::<Input>(&line)?;
        state.turn(input);

        eprintln!("-------------------------  Turn {}", turn_count);
        turn(&mut state, now);
        turn_count += 1;
    }

    Ok(())
}
