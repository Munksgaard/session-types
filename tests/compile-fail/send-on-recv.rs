extern crate session_types;

use std::thread::spawn;
use std::sync::mpsc::channel;

use session_types::*;

type Proto = Send<u8, Eps>;

fn srv(c: Chan2<'static, (), Proto>) {
    c.send(42).close();
}

fn cli(c: Chan2<'static, (), <Proto as HasDual>::Dual>) {
    c.send(42).close(); //~ ERROR
}

fn main() {
    let (c1, c2) = session_channel();
    let t1 = spawn(|| { srv(c1) });
    cli(c2);
    t1.join().unwrap();
}
