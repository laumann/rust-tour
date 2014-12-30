/// Simple channels example
///
///
use std::thread::Thread;

static NTASKS: uint = 10;

fn main() {
    let mut senders = Vec::new();

    for _ in range(0, NTASKS) {
	let (tx, rx) = channel();
	senders.push(tx);
	Thread::spawn(move|| {
	    let n = rx.recv();
	    println!("task received {}", n);
	}).detach();
    }

    for i in range(0, NTASKS) {
	senders[i].send(i);
    }
}
