/// generic.rs
///
/// This example demonstrates how we can use traits to send values through a
/// channel without actually knowing the type of the value.

extern crate session_types;
use session_types::*;

use std::thread::spawn;

fn srv<'run, A: std::marker::Send+'static>(x: A, c: Chan2<'run, (), Send<A, Eps>>) -> Complete<'run, Send<A, Eps>> {
    return c.send(x).close();
}

fn cli<'run, A: std::marker::Send+std::fmt::Debug+'static>(c: Chan2<'run, (), Recv<A, Eps>>) -> Complete<'run, Recv<A, Eps>> {
    let (c, x) = c.recv();
    println!("{:?}", x);
    return c.close();
}

fn main() {
    let (c1, c2) = session_channel();
    let t = spawn(move || srv(42u8, c1));
    cli(c2);

    t.join().unwrap();
}
