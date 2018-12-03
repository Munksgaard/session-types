extern crate session_types;

use session_types::*;

fn main() {
    let (tcs, rcs) = session_channel();
    let (tcu, rcu) = session_channel();

    let receivers = vec!(rcs, rcu);

    let () = tcs.send("Hello, World from TCS!".to_string()).close();

    let (ready, mut rest) = hselect(receivers);

    let (to_close, s) = ready.recv();
    println!("Got a response: \"{}\"", s);
    to_close.close();

    let () = tcu.send("Hello, World from TCU!".to_string()).close();

    let () = rest
        .drain(..)
        .for_each(|r| {
            let (to_close, s) = r.recv();
            println!("Also got this: \"{}\"", s);
            to_close.close()
        });
}
