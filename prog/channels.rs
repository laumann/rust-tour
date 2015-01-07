/// Simple channels example
///
/// Start ten channels
use std::thread::Thread;
use std::sync::mpsc::channel;

static NTASKS: uint = 10;

fn main() {
    let mut txs = Vec::with_capacity(NTASKS);
    let mut threads = Vec::with_capacity(NTASKS);

    for _ in range(0, NTASKS) {
        let (tx, rx) = channel();
        txs.push(tx);

        let join_guard = Thread::scoped(move|| {
            let i = rx.recv().unwrap();
            println!("{}", rx.recv().unwrap());
        });
        threads.push(join_guard);
    }

    for (i, tx) in txs.iter().enumerate() {
        tx.send(i).unwrap();
    }

    for t in threads.into_iter() {
        t.join();
    }
}
