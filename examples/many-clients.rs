#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
extern crate rand;
extern crate session_types;

use rand::random;
use session_types::*;
use std::thread::spawn;

type Server = Recv<u8, Choose<Send<u8, Eps>, Eps>>;
type Client = <Server as HasDual>::Dual;

fn server_handler(c: Chan<(), Server>) {
    let (c, n) = c.recv();
    match n.checked_add(42) {
        Some(n) => c.sel1().send(n).close(),
        None => c.sel2().close(),
    }
}

/// A channel on which we will receive channels
///
type ChanChan = Offer<Eps, Recv<Chan<(), Server>, Var<Z>>>;

/// server sits in a loop accepting session-typed channels. For each received channel, a new thread
/// is spawned to handle it.
///
/// When the server is asked to quit, it returns how many connections were handled
fn server(rx: Chan<(), Rec<ChanChan>>) -> usize {
    let mut count = 0;
    let mut c = rx.enter();
    loop {
        c = offer! { c,
            Quit => {
                c.close();
                break
            },
            NewChan => {
                let (c, new_chan) = c.recv();
                spawn(move || server_handler(new_chan));
                count += 1;
                c.zero()
            }
        }
    }
    count
}

fn client_handler(c: Chan<(), Client>) {
    let n = random();
    match c.send(n).offer() {
        Left(c) => {
            let (c, n2) = c.recv();
            c.close();
            println!("{} + 42 = {}", n, n2);
        }
        Right(c) => {
            c.close();
            println!("{} + 42 is an overflow :(", n);
        }
    }
}

fn main() {
    let (tx, rx) = session_channel();

    let n: u8 = random();
    let mut tx = tx.enter();

    println!("Spawning {} clients", n);
    let mut ts = vec![];
    for _ in 0..n {
        let (c1, c2) = session_channel();
        ts.push(spawn(move || {
            client_handler(c2);
        }));
        tx = tx.sel2().send(c1).zero();
    }
    tx.sel1().close();
    let count = server(rx);
    for t in ts {
        let _ = t.join();
    }
    println!("Handled {} connections", count);
}
