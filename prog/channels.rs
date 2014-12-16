/// Simple channels example
///
///

static NTASKS: uint = 10;

fn main() {
    let mut senders = Vec::new();

    for _ in range(0, NTASKS) {
	let (tx, rx) = channel();
	senders.push(tx);
	spawn(move|| {
	    let n = rx.recv();
	    println!("task received {}", n);
	});
    }

    for i in range(0, NTASKS) {
	senders[i].send(i);
    }
}
