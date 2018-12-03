/// This is an implementation of an echo server.

/// One process reads input and sends it to the other process, which outputs it.
extern crate session_types;
use session_types::*;

use std::thread::spawn;

type Srv = Offer<Eps, Recv<String, Var<Z>>>;
fn srv(c: Chan<(), Rec<Srv>>) {
    let mut c = c.enter();

    loop {
        c = offer!{ c,
            CLOSE => {
                println!("Closing server.");
                c.close();
                break
            },
            RECV => {
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
    let mut buf = String::with_capacity(1024);
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
    println!("Starting echo server. Press 'q' to quit.");
    let t = spawn(move || srv(c1));
    cli(c2);
    let _ = t.join();
}
