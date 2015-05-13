extern crate session_types;
extern crate rand;

use session_types::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use rand::random;

type Server = Recv<u8, Choose<Send<u8, Eps>, Eps>>;
type Client = Send<u8, Offer<Recv<u8, Eps>, Eps>>;

fn handler(c: Chan<(), Server>) {
    let (c, n) = c.recv();
    match n.checked_add(42) {
        Some(n) => c.sel1().send(n).close(),
        None => c.sel2().close(),
    }
}

fn server(rx: Receiver<Chan<(), Server>>) {
    let mut count = 0;
    loop {
        match borrow_request(&rx) {
            Some(c) => {
                spawn(move || handler(c));
                count += 1;
            },
            None => break,
        }
    }
    println!("Handled {} connections", count);
}

fn client(tx: Sender<Chan<(), Server>>) {
    let c = accept(tx).unwrap();

    let n = random();
    match c.send(n).offer() {
        Ok(c) => {
            let (c, n2) = c.recv();
            c.close();
            println!("{} + 42 = {}", n, n2);
        },
        Err(c) => {
            c.close();
            println!("{} + 42 is an overflow :(", n);
        }
    }
}

fn main() {
    let (tx, rx) = channel();
    let mut buf = Vec::new();

    let n: u8 = random();
    println!("Spawning {} clients", n);
    for _ in 0..n {
        let tmp = tx.clone();
        buf.push(spawn(move || client(tmp)));
    }
    drop(tx);

    server(rx);
    for t in buf {
        t.join().unwrap();
    }
}
