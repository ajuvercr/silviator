use std::ops::Index;

use super::*;
use crate::models::Planet;

#[derive(Clone, Copy, Default, Debug)]
struct ExpEvent {
    ships: i32,
}

impl ExpEvent {
    fn reset(&mut self) {
        self.ships = 0;
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct PlanetState {
    owner: Owner,
    ships: i32,
}

impl PlanetState {
    fn next(&self) -> Self {
        if self.owner == NEUTRAL {
            *self
        } else {
            Self {
                owner: self.owner,
                ships: self.ships + 1,
            }
        }
    }
}

#[derive(Debug)]
pub struct PlanetStates {
    changed: bool,
    pub planet: Planet,
    states: VecDeque<Vec<ExpEvent>>,
    future: VecDeque<Planet>,
}

fn new_state(player: usize) -> Vec<ExpEvent> {
    let mut events = Vec::with_capacity(player);
    events.resize(player, ExpEvent::default());
    events
}

fn execute_combat(current: PlanetState, exp_events: &Vec<ExpEvent>) -> PlanetState {
    let mut bigest = (NEUTRAL, 0);
    let mut second = (NEUTRAL, 0);

    for (i, x) in exp_events.iter().enumerate() {
        let count = if i == current.owner {
            current.ships + x.ships
        } else {
            x.ships
        };

        if count > bigest.1 {
            second = bigest;
            bigest = (i, count);
        } else if count > second.1 {
            second = (i, count);
        }
    }

    if bigest.1 == second.1 {
        PlanetState {
            ships: 0,
            owner: NEUTRAL,
        }
    } else {
        PlanetState {
            ships: bigest.1 - second.1,
            owner: bigest.0,
        }
    }
}

#[allow(unused)]
impl PlanetStates {
    pub fn new(planet: Planet, players: usize, max_size: usize) -> Self {
        let mut out = Self {
            changed: false,
            planet,
            states: VecDeque::with_capacity(max_size),
            future: VecDeque::with_capacity(max_size + 1),
        };

        out.states.resize(max_size, new_state(players + 1));
        out.future.resize(max_size + 1, out.planet);

        out.calculate_states(planet).unwrap();

        out
    }

    pub fn id(&self) -> usize {
        self.planet.id
    }

    pub fn distance(&self, other: &Self) -> usize {
        (*self.planet.loc() - *other.planet.loc()).length().ceil() as usize
    }

    pub fn futures(&self) -> impl Iterator<Item = &Planet> {
        self.future.iter()
    }

    pub fn incoming_exp(&mut self, expedition: &Expedition) {
        assert_eq!(expedition.destination, self.planet.id);

        self.states[expedition.remaining][expedition.owner].ships += expedition.ships;
        self.changed = true;
    }

    pub fn dispatch(&mut self, ship_count: i32) {
        self.planet.ships -= ship_count;
        self.changed = true;
    }

    pub fn flush(&mut self, planet: Planet) {
        if self.changed {
            self.changed = false;
            self.calculate_states(planet).unwrap();
        }
    }

    pub fn turn(&mut self) {
        if self.planet.owner != NEUTRAL {
            self.planet.ships += 1;
        }

        self.states.rotate_left(1);
        if let Some(st) = self.states.back_mut() {
            let mut current = PlanetState {
                owner: self.planet.owner,
                ships: self.planet.ships,
            };
            current = execute_combat(current, st);
            self.planet.owner = current.owner;
            self.planet.ships = current.ships;

            st.iter_mut().for_each(|y| y.reset());
        }

        self.changed = true;
    }

    fn calculate_states(&mut self, planet: Planet) -> Option<()> {
        if planet.owner != self.planet.owner || planet.ships != self.planet.ships {
            eprintln!("Got something looking like a zero turn move");
        }
        self.planet.ships = planet.ships;
        self.planet.owner = planet.owner;

        let mut current = PlanetState {
            owner: planet.owner,
            ships: planet.ships,
        };

        // First state is self
        self.future[0] = self.planet;

        for (state, future) in self.states.iter().zip(self.future.iter_mut().skip(1)) {
            // construction
            current = current.next();

            current = execute_combat(current, state);
            // Arrival (owner, count)

            *future = Planet {
                ships: current.ships,
                owner: current.owner,
                id: self.planet.id,
                loc: self.planet.loc,
            };
        }

        Some(())
    }
}

impl Index<usize> for PlanetStates {
    type Output = Planet;

    fn index(&self, index: usize) -> &Self::Output {
        &self.future[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(ships: i32, owner: usize) -> Planet {
        Planet {
            id: 0,
            ships,
            owner,
            loc: Vec2::new(0., 0.),
        }
    }

    fn e(ships: i32, remaining: usize, owner: usize) -> Expedition {
        Expedition {
            id: 0,
            origin: 0,
            destination: 0,
            ships,
            remaining,
            owner,
        }
    }

    #[test]
    fn simple_test() {
        let p0 = |i| p(i, 0);
        let p1 = |i| p(i, 1);

        let mut player = PlanetStates::new(p1(0), 2, 5);
        player.flush(p1(0));
        assert_eq!(
            player.future,
            vec![p1(0), p1(1), p1(2), p1(3), p1(4), p1(5)]
        );

        player.turn();
        player.flush(p1(1));
        assert_eq!(
            player.future,
            vec![p1(1), p1(2), p1(3), p1(4), p1(5), p1(6)]
        );

        let mut neutral = PlanetStates::new(p0(0), 2, 5);
        neutral.flush(p0(0));
        assert_eq!(
            neutral.future,
            vec![p0(0), p0(0), p0(0), p0(0), p0(0), p0(0)]
        );
    }

    #[test]
    fn test_with_ships_one() {
        let p1 = |i| p(i, 1);

        let mut ps = PlanetStates::new(p1(0), 2, 5);
        let exp = e(2, 2, 2);

        ps.incoming_exp(&exp);
        ps.flush(p1(0));

        assert_eq!(ps.future, vec![p1(0), p1(1), p1(2), p1(1), p1(2), p1(3)]);

        ps.turn();
        ps.flush(p1(1));
        assert_eq!(ps.future, vec![p1(1), p1(2), p1(1), p1(2), p1(3), p1(4)]);
    }

    #[test]
    fn test_with_ships_with_conquering() {
        let p1 = |i| p(i, 1);
        let p2 = |i| p(i, 2);

        let mut ps = PlanetStates::new(p1(0), 2, 5);
        let exp = e(5, 2, 2);

        ps.incoming_exp(&exp);
        ps.flush(p1(0));

        assert_eq!(ps.future, vec![p1(0), p1(1), p1(2), p2(2), p2(3), p2(4)]);
    }

    #[test]
    fn test_with_dispatch() {
        let p1 = |i| p(i, 1);

        let mut ps = PlanetStates::new(p1(2), 2, 5);
        ps.dispatch(2);
        ps.flush(p1(0));

        assert_eq!(ps.future, vec![p1(0), p1(1), p1(2), p1(3), p1(4), p1(5)]);

        ps.turn();
        ps.flush(p1(1));
        assert_eq!(ps.future, vec![p1(1), p1(2), p1(3), p1(4), p1(5), p1(6)]);
    }
}
