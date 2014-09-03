use std::rand;
use std::rand::distributions::{IndependentSample,Range};
use std::io::timer::sleep;

use std::time::Duration;

static nrequesters: uint = 10;
static nworkers: uint = 10;

struct Request {
    work: uint,        // The worker function to execute
}

fn requester(q: Sender<Request>) {
    let range = Range::new(0u, 5000);
    let mut rng = rand::task_rng();
    loop {
        let dur = range.ind_sample(&mut rng);
        sleep(Duration::milliseconds(dur as i64));

        q.send(Request{work: dur});
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
                let w = workers.get_mut(0);
                w.requests.send(req);
                w.pending += 1;
            },
            id = done.recv() => {
                for w in workers.mut_iter() {
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
        sleep(Duration::milliseconds((req.work << 1) as i64));
        
        done.send(id);
    }
}

/// We have n requesters, m workers and one dispatcher.
///
///
fn main() {
    let (tx, rx) = channel();

    for _ in range(0, nrequesters) {
        let qc = tx.clone();
        spawn(proc() requester(qc));
    }

    let mut workers = Vec::with_capacity(nworkers);

    let (tx_done, rx_done) = channel();
    for id in range(0, nworkers) {
        let (wtx, wrx) = channel();
        let txd = tx_done.clone();

        let w = Worker{id: id, pending: 0, requests: wtx};
        workers.push(w);

        spawn(proc() worker(id, wrx, txd));
    }

    // start the dispatcher
    spawn(proc() dispatch(rx, workers, rx_done));
}
