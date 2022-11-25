use std::{collections::HashMap, fmt::Display};

use vecs::Vec2;

use super::{Expedition, Input, MoveOutput, Output, Planet, PlanetInput, PlanetStates};

struct PlanetFmt<'a> {
    state: &'a State,
    planet: &'a Planet,
}

impl<'a> Display for PlanetFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} owner {} ship_count {}",
            self.state.inv_planet_map[self.planet.id], self.planet.owner, self.planet.ships
        )
    }
}

type PlanetMap = HashMap<String, usize>;

pub struct State {
    planets: Vec<PlanetStates>,

    planet_map: PlanetMap,
    inv_planet_map: Vec<String>,

    handled_exps: u64,

    turns: Vec<(usize, usize, i32)>,
}

fn map_planet(p: &PlanetInput, map: &PlanetMap) -> Planet {
    Planet {
        id: map[&p.name],
        ships: p.ship_count,
        owner: p.owner.unwrap_or_default(),
        loc: Vec2::new(p.x, p.y),
    }
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

        eprintln!("Max dist {}", max_dist);

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
        }
    }

    pub fn turn(
        &mut self,
        Input {
            planets,
            expeditions,
        }: Input,
    ) {
        let exps = self.handled_exps;

        expeditions
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
            .for_each(|e| {
                eprintln!("Handling expedition {}", e.id);
                self.planets[e.destination].incoming_exp(&e);
                self.planets[e.origin].dispatch(e.ships);
                self.handled_exps += 1;
            });

        self.planets.iter_mut().for_each(|p| {
            p.turn();
            p.flush()
        });

        // Testing if inner state is still valid
        planets
            .iter()
            .map(|p| map_planet(p, &self.planet_map))
            .for_each(|p| {
                if self.planets[p.id][0] != p {
                    eprintln!(
                        "Planets did not align\nexp {}\ngot    {}",
                        self.fmt(&p),
                        self.fmt(&self.planets[p.id][0])
                    )
                }
            });
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
