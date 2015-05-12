extern crate rust_sessions;

use std::thread::spawn;
use std::sync::mpsc::channel;

use rust_sessions::*;

type Proto = Send<u8, Eps>;

fn main() {
    let (tx, rx) = channel();
    let guard = spawn(|| {
        let c: Chan<(), Proto> = accept(tx).unwrap();
        c.send(42).close();
    });
    let c: Chan<(), Proto> = rx.recv().ok().unwrap(); // Does not use request!!
    //~^ ERROR mismatched types
    c.send(42).close();
}
