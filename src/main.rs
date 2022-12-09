#![feature(fn_traits)]
#![feature(unboxed_closures)]
use std::{error::Error, io::stdin};

use crate::models::*;

use std::io::BufRead;
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
struct OptionalMove {
    source: usize,
    target: usize,
    ships: i32,

    duration: usize,
}

fn calculate_attack_moves(friendlies: &mut Vec<Planet>, target: &Planet) -> Vec<OptionalMove> {
    let mut out = Vec::new();
    let ord = PlanetOrderer::planet(target);
    friendlies.sort_by(ord);

    let mut ships_required = target.ships;
    for friendly in friendlies {
        if ships_required < 0 {
            break;
        }

        let sc = friendly.ships;
        let duration = (friendly.loc - target.loc).length();

        out.push(OptionalMove {
            source: friendly.id,
            target: target.id,
            ships: sc,
            duration: duration.ceil() as usize,
        });
        ships_required -= sc;
    }

    if ships_required >= 0 {
        return Vec::new();
    }

    out
}

fn best_planet(state: &mut State) -> Option<()> {
    let friendlies: Vec<_> = state
        .planets()
        .iter()
        .map(|p| p[0])
        .filter(|x| x.owner == ME)
        .collect();

    let turnes = state
        .enemies_at(10)
        .filter_map(|enemy| {
            let attack_moves = calculate_attack_moves(&mut friendly, &enemy);
            eprintln!("Attack moves {:?}", attack_moves);
            (!attack_moves.is_empty()).then_some(attack_moves)
        })
        .min_by_key(|x| x.iter().map(|y| y.duration).max().unwrap())?;

    let max_turn = turnes.iter().map(|y| y.duration).max().unwrap();
    for turn in turnes {
        if turn.duration == max_turn {
            state.add_turn(turn.source, turn.target, turn.ships);
        }
    }

    Some(())
}

fn turn(state: &mut State) {
    simple_turn(state);

    let output = state.flush();
    println!("{}", output);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut lines = stdin().lock().lines();

    let line = lines.next().unwrap()?;
    let input = serde_json::from_str::<Input>(&line).unwrap();
    let mut state = State::new(input);

    let mut turn_count = 0;

    eprintln!("-------------------------  Turn {}", turn_count);
    turn(&mut state);
    turn_count += 1;

    while let Some(line) = lines.next() {
        let input = serde_json::from_str::<Input>(&line?)?;
        state.turn(input);

        eprintln!("-------------------------  Turn {}", turn_count);
        turn(&mut state);
        turn_count += 1;
    }

    Ok(())
}
