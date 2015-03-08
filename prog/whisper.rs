/*
 * Chinese whispers in Rust
 *
 * Start a number of processes passing along an uint. The whisper "distortion"
 * is an increment on the passed along uint.
 *
 * The workings can be sketched in the following way:
 *
 *          p1           p2          p3          p4
 *      +---------+ +---------+ +---------+ +---------+
 * (tx ,|rx) - (tx|,|rx) - (tx|,|rx) - (tx|,|rx) - (tx|,|rx)
 *   ^  +---------+ +---------+ +---------+ +---------+   |
 *   |       +1          +1          +1          +1       v
 *  in                                                   out
 *
 * Bonus instructive exercise: Try moving the `tx.send(0)` statement above the
 * the for-loop in main(). Then processes can start passing along their data
 * as soon as the proc is invoked.
 */

extern crate getopts;

use getopts::{optopt, optflag, getopts, usage};
use std::os;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;

static NPROC_DEFAULT: usize = 25_000;

/*
 * Short-hand macro for spawn(proc() ...)
 */
macro_rules! go(
    ($e:expr) => (spawn(move|| $e))
);

fn whisper(rx: Receiver<usize>, tx: Sender<usize>) {
    tx.send(rx.recv().unwrap()+1).unwrap();
}

fn main() {
    let nproc = match handle_args() {
        None => return,
        Some(n) => n
    };

    println!("Spawning {} processes.", nproc);

    let (tx, mut rx) = channel();

    for _ in range(0, nproc) {
        let (tx_next, rx_next) = channel();
        go!{ whisper(rx, tx_next) };
        rx = rx_next;
    }

    tx.send(0).unwrap();
    let n = rx.recv().unwrap();
    println!("Received {}", n);
}



/*
 * Handle command-line arguments
 *
 * Return the number of processes to start and whether or not parsing was
 * successful.
 *
 * The tuple returned is inspired by Go's multiple return values (this in
 * essence just models that). I suppose in idiomatic Rust, one should return
 * an Option<usize> indicating whether or not a number could be found.
 *
 * This code was split into its own function, because it started to dominate
 * the code that is actually interesting here. This code should just provide
 * the option to specify the number of processes to start.
 */
fn handle_args() -> Option<usize> {
    let args = os::args();
    let prog = args[0].clone();
    let opts = [
        optopt("n", "", "Length of the whisper chain", "<usize>"),
        optflag("h", "help", "Print this help message")
    ];

    let matches = match getopts(os::args().tail(), &opts) {
        Ok(m)  => m,
        Err(e) => {
            print!("{}", e.to_string());
            print!("{} [options]{}", prog, usage("", &opts));
            return None
        }
    };

    if matches.opt_present("h") {
        print!("{} [options]{}", prog, usage("", &opts));
        return None
    }

    if matches.opt_present("n") {
        match std::num::from_str_radix(matches.opt_str("n").unwrap().as_slice(), 10) {
            Ok(n)  => Some(n),
            Err(_) => {
                println!("error: argument for -n must be positive numeric.");
                print!("{} [options]{}", prog, usage("", &opts));
                return None
            }
        }
    } else {
        Some(NPROC_DEFAULT)
    }
}
