/// Monty hall
///
/// Taken from http://www.jonashietala.se/blog
///
/// Expanded with optional argument on num_iterations

extern crate getopts;

use std::os;
use std::rand;
use std::rand::Rng;
use std::rand::distributions::{IndependentSample,Range};

use std::uint::parse_bytes;
use getopts::{optopt,getopts,usage};

struct SimulationResult {
    win:      bool,
    switched: bool
}

fn simulate<R: Rng>(random_door: &Range<uint>, rng: &mut R) -> SimulationResult {
    let car = random_door.ind_sample(rng);
    let mut choice = random_door.ind_sample(rng);

    let open = game_host_open(car, choice, rng);

    let switch = rng.gen();
    if switch {
        choice = switch_door(choice, open);
    }
    SimulationResult{win: choice == car, switched: switch}
}

fn game_host_open<R: Rng>(car: uint, choice: uint, rng: &mut R) -> uint {
    let choices = free_doors(&[car, choice]);
    rand::sample(rng, choices.move_iter(), 1)[0]
}

fn switch_door(choice: uint, open: uint) -> uint {
    free_doors(&[choice, open])[0]
}

fn free_doors(blocked: &[uint]) -> Vec<uint> {
    range(0u, 3).filter(|x| !blocked.contains(x)).collect()
}

fn main() {
    // Option parsing
    let opts = [
        optopt("n", "num-simulations", "Set the number of iterations to run", "<uint>"),
    ];

    let matches = match getopts(os::args().tail(), opts) {
        Ok(m)  => m,
        Err(f) => fail!(f.to_string())
    };

    let num_simulations = if matches.opt_present("n") {
        let n = matches.opt_str("n").unwrap();
        match parse_bytes(n.as_bytes(), 10) {
            Some(n) => n,
            None    => {
                println!("error: Argument to -n: '{}'. Must be an unsigned integer", n);
                println!("{}", usage("\nusage: ./monty_hall [options]", opts));
                return;
            }
        }
    } else { 1000u };


    // Now for the simulation
    let mut rng = rand::task_rng();
    let random_door = Range::new(0u,3);

    let (mut switch_wins, mut switch_losses) = (0u, 0u);
    let (mut keep_wins, mut keep_losses) = (0u, 0u);

    println!("Running {} simulations...", num_simulations);

    for _ in range(0u, num_simulations) {
        let result = simulate(&random_door, &mut rng);

        match (result.win, result.switched) {
            (true, true)  => switch_wins += 1,
            (true, false) => keep_wins += 1,
            (false, true) => switch_losses += 1,
            _             => keep_losses += 1,
        }
    }

    let total_switches = switch_wins + switch_losses;
    let total_keeps = keep_wins + keep_losses;

    println!("Switched door {} times with {} wins and {} losses", total_switches, switch_wins, switch_losses);
    println!("Kept door {} times with {} wins and {} losses", total_keeps, keep_wins, keep_losses);
    
    println!("Estimated chance to win if we switch: {}", switch_wins as f32 / total_switches as f32);
    println!("Estimated chance to win if we keep:   {}", keep_wins as f32 / total_keeps as f32);
}