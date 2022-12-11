use std::{collections::HashMap, fmt::Display};

use vecs::Vec2;

use super::{Expedition, Input, MoveOutput, Output, Owner, Planet, PlanetInput, PlanetStates};

struct PlanetFmt<'a> {
    state: &'a State,
    planet: &'a Planet,
}

impl<'a> Display for PlanetFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Planet{{\"{}\", owner: {} ship_count: {}}}",
            self.state.inv_planet_map[self.planet.id], self.planet.owner, self.planet.ships
        )
    }
}

struct ExpeditionFmt<'a> {
    state: &'a State,
    exp: &'a Expedition,
}
impl<'a> Display for ExpeditionFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let origin = &self.state.inv_planet_map[self.exp.origin];
        let destination = &self.state.inv_planet_map[self.exp.destination];
        write!(
            f,
            "Expedition{{{}, owner: {}, turns: {}, origin: {:?}, destination: {:?}, ship_count {}}}",
            self.exp.id, self.exp.owner, self.exp.remaining, origin, destination, self.exp.ships
        )
    }
}

type PlanetMap = HashMap<String, usize>;

fn map_planet(p: &PlanetInput, map: &PlanetMap) -> Planet {
    Planet {
        id: map[&p.name],
        ships: p.ship_count,
        owner: p.owner.unwrap_or_default(),
        loc: Vec2::new(p.x, p.y),
    }
}

pub struct State {
    planets: Vec<PlanetStates>,

    planet_map: PlanetMap,
    inv_planet_map: Vec<String>,

    handled_exps: u64,

    turns: Vec<(usize, usize, i32)>,
    turn: usize,
}

impl State {
    pub fn new(input: Input) -> Self {
        let mut planets = Vec::new();
        let mut planet_map = HashMap::new();
        let mut inv_planet_map = Vec::new();

        for p in input.planets {
            let id = inv_planet_map.len();
            planet_map.insert(p.name.clone(), id);
            inv_planet_map.push(p.name.clone());

            planets.push(Planet {
                id,
                ships: p.ship_count,
                owner: p.owner.unwrap_or_default(),
                loc: Vec2::new(p.x, p.y),
            });
        }

        let max_dist = planets
            .iter()
            .flat_map(|p1| planets.iter().map(|p2| (p1.loc - p2.loc).length()))
            .max_by(|x, y| x.total_cmp(&y))
            .unwrap()
            .ceil() as usize;

        let min_dist = planets
            .iter()
            .flat_map(|p1| {
                planets
                    .iter()
                    .filter(|x| p1.id != x.id)
                    .map(|p2| (p1.loc - p2.loc).length())
            })
            .min_by(|x, y| x.total_cmp(&y))
            .unwrap()
            .ceil() as usize;

        eprintln!("Test distances max {} min {}", max_dist, min_dist);

        let planet_count = planets.len();
        let planet_states = planets
            .into_iter()
            .map(|p| PlanetStates::new(p, planet_count, max_dist))
            .collect();

        Self {
            planets: planet_states,
            planet_map,
            inv_planet_map,
            handled_exps: 0,
            turns: Vec::new(),
            turn: 0,
        }
    }

    pub fn type_at<F: Fn(&Owner) -> bool + 'static>(
        &self,
        fut: usize,
        is_owner: F,
    ) -> impl Iterator<Item = &PlanetStates> {
        self.planets()
            .iter()
            .filter(move |x| is_owner(&x[fut].owner))
    }

    pub fn turn(
        &mut self,
        Input {
            planets,
            expeditions,
        }: Input,
    ) {
        self.turn += 1;
        eprintln!("{} -----------------------", self.turn);
        let exps = self.handled_exps;

        for e in expeditions
            .into_iter()
            .filter(|e| e.id >= exps)
            .map(|e| Expedition {
                id: e.id,
                ships: e.ship_count,
                owner: e.owner,
                remaining: e.turns_remaining,
                origin: self.planet_map[&e.origin],
                destination: self.planet_map[&e.destination],
            })
        {
            eprintln!(
                "Handling expedition {}",
                ExpeditionFmt {
                    state: self,
                    exp: &e
                }
            );
            self.planets[e.destination].incoming_exp(&e);
            self.planets[e.origin].dispatch(e.ships);
            self.handled_exps += 1;
        }

        for planet in planets {
            let idx = self.planet_map[&planet.name];
            let p = &mut self.planets[idx];
            p.turn(map_planet(&planet, &self.planet_map));
            p.flush()
        }
    }

    fn fmt<'a>(&'a self, p: &'a Planet) -> PlanetFmt<'a> {
        PlanetFmt {
            state: self,
            planet: p,
        }
    }

    pub fn planets(&self) -> &[PlanetStates] {
        &self.planets
    }

    pub fn add_turn(&mut self, source: usize, target: usize, ships: i32) {
        self.turns.push((source, target, ships));
    }

    pub fn flush(&mut self) -> String {
        let moves: Vec<_> = self
            .turns
            .drain(..)
            .map(|(source, target, count)| {
                let origin = &self.inv_planet_map[source];
                let destination = &self.inv_planet_map[target];

                MoveOutput {
                    origin,
                    destination,
                    ship_count: count,
                }
            })
            .collect();

        let output = Output { moves };

        serde_json::to_string(&output).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Input;

    use super::State;

    fn test_states(input: &str) {
        let turns: Vec<Input> = input
            .lines()
            .map(|x| serde_json::from_str(x).unwrap())
            .collect();

        let mut state: Option<State> = None;
        for turn in turns {
            if let Some(state) = state.as_mut() {
                // assert!(state.turn(turn));
            } else {
                state = Some(State::new(turn));
            }
        }
    }

    #[test]
    fn test_hex() {
        test_states(include_str!("../../tests/success_hex.txt"));
    }

    #[test]
    fn test_one() {
        test_states(include_str!("../../tests/fail_hungergames.txt"));
    }

    #[test]
    fn test_spiral() {
        test_states(include_str!("../../tests/success_spiral.txt"));
    }
}
