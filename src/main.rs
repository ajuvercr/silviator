#![feature(fn_traits)]
#![feature(unboxed_closures)]
use std::{
    collections::BinaryHeap,
    error::Error,
    io::stdin,
    time::{Duration, Instant},
};

use graphs::Operation;

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
        self.weight / (self.required_ships.pow(2) as f32 + self.duration.pow(2) as f32)
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

        this_score.partial_cmp(&other_score).unwrap()
    }
}

fn find_optional_operations(
    target: &PlanetStates,
    states: &State,
    queue: &mut BinaryHeap<OptionalOperation>,
) {
    let weight = if target.planet.owner == ME { 2. } else { 1. };
    let mut distances: Vec<Vec<(&PlanetStates, usize)>> = Vec::new();

    for p in states.planets() {
        let d = p.distance(target);
        while distances.len() <= d {
            distances.push(Vec::new());
        }
        distances[d].push((p, d));
    }

    let mut options = Vec::new();
    for (d, extra_options) in distances.into_iter().enumerate() {
        if extra_options.is_empty() {
            continue;
        }

        options.extend(extra_options);

        if target[d].owner == ME {
            continue;
        }

        let required_ships = target[d].ships + 1;
        let mut usable_planets = Vec::new();

        for (o, actual_dist) in &options {
            let optional_planet = o[d - actual_dist];
            if optional_planet.owner != ME {
                continue;
            }

            usable_planets.push(UsablePlanet {
                id: o.id(),
                dist: *actual_dist,
                usable_ships: optional_planet.ships,
            });
        }

        if usable_planets.iter().map(|x| x.usable_ships).sum::<i32>() > required_ships {
            let oo = OptionalOperation {
                weight,
                duration: d,
                required_ships,
                usable_planets,
                target: target.id(),
            };

            queue.push(oo);
            return;
        }
    }
}

fn print_operations(state: &State, operation: &Vec<Operation>) {
    for operation in operation {
        eprintln!("Operation:");
        for part in &operation.solution {
            eprintln!(
                "  {} --{}-> {}",
                state.inv_planet_map[part.source], part.ships, state.inv_planet_map[part.target]
            );
        }
    }
}

fn best_planet(state: &mut State, started: Instant) -> Option<()> {
    let mut b_heap = BinaryHeap::new();
    state
        .planets()
        .iter()
        .for_each(|p| find_optional_operations(p, state, &mut b_heap));

    let mut best: Vec<Operation> = Vec::new();
    let mut best_score = f32::MIN;

    let mut tried = Vec::new();
    let mut max_len = 0;
    while started.elapsed() < Duration::from_millis(800) {
        if let Some(turn) = b_heap.pop() {
            tried.push(turn);
            let o = try_oo(&tried, state);

            let score = o.iter().map(|x| x.score).sum();
            max_len = max_len.max(o.len());
            if score > best_score {
                eprintln!("Found better");
                print_operations(state, &o);
                best_score = score;
                best = o;
            }
        } else {
            break;
        }
    }

    eprintln!(
        "Executing {} operations with total score {}",
        best.len(),
        best_score
    );

    for operation in best {
        for part in operation.solution {
            state.add_turn(part.source, part.target, part.ships);
        }
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

        eprintln!("-------------------------  Turn {}", turn_count);
        state.turn(input);
        eprintln!("-------------------------");

        turn(&mut state, now);
        turn_count += 1;
    }

    Ok(())
}
