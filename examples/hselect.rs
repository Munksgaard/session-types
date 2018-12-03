extern crate session_types;

use std::thread::spawn;
use std::borrow::ToOwned;
use session_types::*;

fn main() {
    let (tcs, rcs) = session_channel();
    let (tcu, rcu) = session_channel();

    let receivers = vec!(rcs, rcu);

    let () = tcs.send("Hello, World from TCS!".to_string()).close();

    let (ready, mut rest) = hselect(receivers);

    let (toClose, s) = ready.recv();
    println!("Got a response: \"{}\"", s);
    toClose.close();

    let () = tcu.send("Hello, World from TCU!".to_string()).close();

    let () = rest
        .drain(..)
        .for_each(|r| {
            let (toClose, s) = r.recv();
            println!("Also got this: \"{}\"", s);
            toClose.close()
        });
}
