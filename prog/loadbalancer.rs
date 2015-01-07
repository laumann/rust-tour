#![deny(unused_imports)]

extern crate getopts;
use getopts::{optopt,optflag,getopts,usage,OptGroup};
use std::os;
use std::thunk::Invoke;

use std::rand;
use std::rand::distributions::{IndependentSample,Range};
use std::io::timer::sleep;
use std::cmp::Ordering;

use std::time::Duration;
use std::sync::mpsc::{Sender,Receiver,channel};
use std::thread::Thread;

static DEFAULT_REQUESTERS: uint = 10;
static DEFAULT_WORKERS: uint = 10;

struct Request {
    work: Box<Invoke<(), uint> + Send + 'static> // The worker function to execute
}

fn requester(q: Sender<Request>) {
    let range = Range::new(0u, 5000);
    let mut rng = rand::thread_rng();
    loop {
        let dur = range.ind_sample(&mut rng);
        sleep(Duration::milliseconds(dur as i64));

        q.send(Request{work: box move|: ()| dur}).unwrap();
    }
}

/// TODO Implement worker sorting using std::collections::priority_queue.
/// In particular, figure out how the dispatcher should increment/decrement
/// the load counter for a given worker.
///
/// One could stick with the vector of workers, and have them organised by id
/// using an 'outside' structure.
fn dispatch(q: Receiver<Request>, mut workers: Vec<Worker>, done: Receiver<uint>) {
    loop {
        select! {
            req = q.recv() => {
                let w = workers.get_mut(0).unwrap();
                w.requests.send(req.unwrap()).unwrap();
                w.pending += 1;
            },
            id = done.recv() => {
                let id = id.unwrap();
                for w in workers.iter_mut() {
                    if w.id == id {
                        w.pending -= 1;
                        break;
                    }
                }
            }
        }
        workers.sort();
        print(&workers);
    }
}

/// A round-robin dispatcher
///
/// Relies on the fact that the returned 'id' is also the position of the worker
/// in the worker list.
fn dispatch_rr(q: Receiver<Request>, mut workers: Vec<Worker>, done: Receiver<uint>) {
    let mut i = 0u;
    let nw = workers.len();
    loop {
        select! {
            req = q.recv() => {
                let w = workers.get_mut(i).unwrap();
                w.requests.send(req.unwrap()).unwrap();
                w.pending += 1;
                i = (i + 1) % nw;
            },
            id = done.recv() => {
                let w = workers.get_mut(id.unwrap()).unwrap();
                w.pending -= 1;
            }
        }
        print(&workers);
    }
}

fn print(workers: &Vec<Worker>) {
    let (mut sum, mut sumsq) = (0, 0);

    for w in workers.iter() {
        sum += w.pending;
        sumsq += w.pending * w.pending;
        print!("{:3}", w.pending);
    }
    let avg = sum as f64 / workers.len() as f64;
    let sig = (sumsq as f64 / workers.len() as f64) - avg*avg;
    println!("  {:4}   {:4}", avg, sig);
}

struct Worker {
    id:       uint,
    pending:  uint,
    requests: Sender<Request> // work sending channel
}

impl Worker {
    fn new(id: uint, rq: Sender<Request>) -> Worker {
        Worker{
            id: id,
            pending: 0,
            requests: rq
        }
    }
}

impl Eq for Worker {}

impl PartialEq for Worker {
    fn eq(&self, other: &Worker) -> bool {
        self.id == other.id
    }
}

impl Ord for Worker {
    fn cmp(&self, other: &Worker) -> Ordering {
        self.pending.cmp(&other.pending)
    }
}

impl PartialOrd for Worker {
    fn partial_cmp(&self, other: &Worker) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A worker has an ID, runs in a loop receiving work, performs it and sends
/// the result back the provided request return channel, then signals the
/// dispatcher that it is ready for more work.
fn worker(id: uint, requests: Receiver<Request>, done: Sender<uint>) {
    loop {
        let req = requests.recv();

        // Simulated work
        let s = req.unwrap().work.invoke(());
        sleep(Duration::milliseconds((s << 1) as i64));

        done.send(id).unwrap();
    }
}

/// We have n requesters, m workers and one dispatcher.
///
///
fn main() {
    let (nrequesters, nworkers, rr) = match handle_args() {
        Some(c) => c,
        None    => return
    };

    let (tx, rx) = channel();

    for _ in range(0, nrequesters) {
        let tx_clone = tx.clone();
        Thread::spawn(move|| requester(tx_clone));
    }

    let mut workers = Vec::with_capacity(nworkers);

    let (tx_done, rx_done) = channel();
    for id in range(0, nworkers) {
        let (wtx, wrx) = channel();
        let txd = tx_done.clone();

        workers.push(Worker::new(id, wtx));

        Thread::spawn(move|| worker(id, wrx, txd));
    }

    // // start the dispatcher
    if rr {
        println!("Using round-robin dispatcher");
        dispatch_rr(rx, workers, rx_done)
    } else {
        dispatch(rx, workers, rx_done)
    }
}

/*
 * Macro for extracting a uint option, and if not specified use a given default
 */
macro_rules! getopt_uint(
    ($prog:ident, $o:ident, $m:ident, $arg:expr, $def:ident) => (
        if $m.opt_present($arg) {
            match $m.opt_str($arg).unwrap().parse::<uint>() {
                // match from_str::<uint>($m.opt_str($arg).unwrap()as_slice()) {
                None => {
                    println!("error: argument for '-{}' must be positive numeric.", $arg);
                    print_opts($prog.as_slice(), &$o);
                    return None
                },
                Some(u) => u
            }
        } else {
            $def
        }
    )
);

fn handle_args() -> Option<(uint, uint, bool)> {
    let args = os::args();
    let prog = args[0].clone();
    let opts = [
        optopt("s", "requesters", "Number of requesters to spawn.", "<uint>"),
        optopt("w", "workers", "Number of workers to spawn.", "<uint>"),
        optflag("r", "round-robin", "Use the round-robin dispatcher (default false)."),
        optflag("h", "help", "Print this help message and exit.")
    ];

    let matches = match getopts(os::args().tail(), &opts) {
        Ok(m) => m,
        Err(e) => {
            print!("{}\n\n", e.to_string());
            print_opts(prog.as_slice(), &opts);
            return None
        }
    };
    if matches.opt_present("h") {
        print_opts(prog.as_slice(), &opts);
        return None
    }

    let nrequesters = getopt_uint!(prog, opts, matches, "s", DEFAULT_REQUESTERS);
    let nworkers = getopt_uint!(prog, opts, matches, "w", DEFAULT_WORKERS);

    Some((nrequesters, nworkers, matches.opt_present("r")))
}

fn print_opts(prog: &str, opts: &[OptGroup]) {
    print!("usage: {} [options]{}", prog, usage("", opts))
}
