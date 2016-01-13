/// This is an implementation of an echo server.

/// One process reads input and sends it to the other process, which outputs it.

#[macro_use]
extern crate session_types;
use session_types::*;

use std::thread::spawn;

type Srv = Offer<(Eps, Recv<String, Var<Z>>)>;
fn srv(c: Chan<(), Rec<Srv>>) {

    let mut c = c.enter();

    loop {
        c = match c.offer() {
            Branch2::B1(c) => {
                println!("Closing server.");
                c.close();
                break
            },
            Branch2::B2(c) => {
                let (c, s) = c.recv();
                println!("Received: {}", s);
                c.zero()
            }
        };
    }
}

type Cli = <Srv as HasDual>::Dual;
fn cli(c: Chan<(), Rec<Cli>>) {

    let stdin = std::io::stdin();
    let mut count = 0usize;

    let mut c = c.enter();
    let mut buf = "".to_string();
    loop {
        stdin.read_line(&mut buf).ok().unwrap();
        if !buf.is_empty() {
            buf.pop();
        }
        match &buf[..] {
            "q" => {
                let c = c.sel2().send(format!("{} lines sent", count));
                c.zero().sel1().close();
                println!("Client quitting");
                break;
            }
            _ => {
                c = c.sel2().send(buf.clone()).zero();
                buf.clear();
                count += 1;
            }
        }
    }
}

fn main() {
    let (c1, c2) = session_channel();
    spawn(move || srv(c1));
    cli(c2);
}
