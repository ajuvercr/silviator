use std::{cmp::Ordering, collections::VecDeque};

use serde::{Deserialize, Serialize};
use vecs::Vec2;

mod planet_states;
mod state;

pub use planet_states::*;
pub use state::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanetInput {
    ship_count: i32,
    x: f32,
    y: f32,
    owner: Option<usize>,
    name: String,
}

#[derive(Clone, Copy)]
pub struct PlanetOrderer {
    from: Vec2<f32>,
}

impl PlanetOrderer {
    pub fn vec2(from: Vec2<f32>) -> Self {
        Self { from }
    }
    pub fn planet(from: &Planet) -> Self {
        Self { from: from.loc }
    }
}

impl FnOnce<(&Vec2<f32>, &Vec2<f32>)> for PlanetOrderer {
    type Output = Ordering;

    extern "rust-call" fn call_once(self, args: (&Vec2<f32>, &Vec2<f32>)) -> Self::Output {
        let x1 = (self.from - *args.0).length();
        let x2 = (self.from - *args.1).length();
        x1.total_cmp(&x2)
    }
}

impl FnMut<(&Vec2<f32>, &Vec2<f32>)> for PlanetOrderer {
    extern "rust-call" fn call_mut(&mut self, args: (&Vec2<f32>, &Vec2<f32>)) -> Self::Output {
        let x1 = (self.from - *args.0).length();
        let x2 = (self.from - *args.1).length();
        x1.total_cmp(&x2)
    }
}

impl FnOnce<(&Planet, &Planet)> for PlanetOrderer {
    type Output = Ordering;

    extern "rust-call" fn call_once(self, args: (&Planet, &Planet)) -> Self::Output {
        let x1 = (self.from - args.0.loc).length();
        let x2 = (self.from - args.1.loc).length();
        x1.total_cmp(&x2)
    }
}

impl FnMut<(&Planet, &Planet)> for PlanetOrderer {
    extern "rust-call" fn call_mut(&mut self, args: (&Planet, &Planet)) -> Self::Output {
        let x1 = (self.from - args.0.loc).length();
        let x2 = (self.from - args.1.loc).length();
        x1.total_cmp(&x2)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Planet {
    pub id: usize,
    pub ships: i32,
    pub owner: Owner,
    pub loc: Vec2<f32>,
}

impl Planet {
    pub fn loc(&self) -> &Vec2<f32> {
        &self.loc
    }
}

impl AsRef<Vec2<f32>> for Planet {
    fn as_ref(&self) -> &Vec2<f32> {
        &self.loc
    }
}

#[derive(Debug, Deserialize)]
pub struct ExpeditionInput {
    id: u64,
    ship_count: i32,
    origin: String,
    destination: String,
    owner: Owner,
    turns_remaining: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Expedition {
    pub id: u64,
    pub ships: i32,
    pub remaining: usize,
    pub owner: Owner,
    pub origin: usize,
    pub destination: usize,
}

#[derive(Debug, Deserialize)]
pub struct Input {
    planets: Vec<PlanetInput>,
    expeditions: Vec<ExpeditionInput>,
}

#[derive(Debug, Serialize)]
pub struct MoveOutput<'a> {
    origin: &'a str,
    destination: &'a str,
    ship_count: i32,
}

#[derive(Debug, Serialize)]
pub struct Output<'a> {
    moves: Vec<MoveOutput<'a>>,
}

pub type Owner = usize;
pub const ME: Owner = 1;
pub const NEUTRAL: Owner = 0;

// pub fn is_me(owner: &Owner) -> bool {
//     *owner == ME
// }
//
// pub fn is_not_me(owner: &Owner) -> bool {
//     *owner != ME
// }
//
// pub fn is_enemy(owner: &Owner) -> bool {
//     *owner > 1
// }
// pub fn is_neutral(owner: &Owner) -> bool {
//     *owner == 0
// }

#[cfg(test)]
mod tests {
    use vecs::Vec2;

    use super::PlanetOrderer;

    macro_rules! v {
        ($x:expr, $y:expr) => {
            Vec2::new($x, $y)
        };
    }

    #[test]
    fn comparator_works() {
        let p0 = v!(0., 0.);
        let p1 = v!(0., 1.);
        let p2 = v!(0., 2.);
        let p3 = v!(0., -3.);

        let mut sorter = vec![p2, p1, p0, p3];
        let cmp = PlanetOrderer::vec2(v!(0., 0.));
        sorter.sort_by(cmp);

        assert_eq!(sorter, vec![p0, p1, p2, p3]);
    }
}
