extern crate session_types;

use std::thread::spawn;
use std::sync::mpsc::channel;

use session_types::*;

fn srv(c: Chan<(), Recv<u8, Eps>>) {
    let (c, x) = c.recv();
    c.close();
}

fn main() {
    let (c1, c2) = session_channel();
    let t1 = spawn(|| { srv(c2) });

    let c1_ = c1;
    c1_.send(42).close();
    c1.send(42).close();        //~ ERROR

    t1.join().unwrap();
}
