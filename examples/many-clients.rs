
extern crate rand;
extern crate session_types;

use session_types::*;
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;
use rand::random;

type Server = Recv<u8, Choose<Send<u8, Eps>, Eps>>;
type Client = <Server as HasDual>::Dual;

fn server_handler(c: Chan<(), Server>) {
    let (c, n) = c.recv();
    match n.checked_add(42) {
        Some(n) => c.sel1().send(n).close(),
        None => c.sel2().close(),
    }
}

fn server(rx: Receiver<Chan<(), Server>>) {
    let mut count = 0;
    loop {
        match rx.recv() {
            Ok(c) => {
                spawn(move || server_handler(c));
                count += 1;
            }
            Err(_) => break,
        }
    }
    println!("Handled {} connections", count);
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
    let (tx, rx) = channel();

    let n: u8 = random();
    println!("Spawning {} clients", n);
    for _ in 0..n {
        let tmp = tx.clone();
        spawn(move || {
            let (c1, c2) = session_channel();
            tmp.send(c1).unwrap();
            client_handler(c2);
        });
    }
    drop(tx);

    server(rx);
}
