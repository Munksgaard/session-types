//! A variation of the echo server where the server wakes up every 20ms
//! and polls for pending messages (single characters in this case): here
//! the "server" echos the received character after converting to uppercase,
//! and the "client" generates a some characters at different intervals.

extern crate session_types;
use session_types::*;

type Term = Eps;

type Upcase = Offer<Recv<char, Var<Z>>, Term>;

type Chargen = <Upcase as HasDual>::Dual;

fn upcase(chan: Chan<(), Rec<Upcase>>) {
    let mut chan = chan.enter();
    'outer: loop {
        println!("upcase: tick!");
        let mut poll = true;
        while poll {
            let result = try_offer! { chan,
                ACHAR => {
                    let (chan, ch) = chan.recv();
                    println!("{:?}", ch.to_uppercase().next().unwrap());
                    Ok (chan.zero())
                },
                QUIT => {
                    chan.close();
                    break 'outer
                }
            };
            chan = match result {
                Ok(chan) => chan,
                Err(chan) => {
                    poll = false;
                    chan
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn chargen(chan: Chan<(), Rec<Chargen>>) {
    let mut chan = chan.enter();
    chan = chan.sel1().send('a').zero();
    chan = chan.sel1().send('b').zero();
    chan = chan.sel1().send('c').zero();
    std::thread::sleep(std::time::Duration::from_millis(100));
    chan = chan.sel1().send('d').zero();
    std::thread::sleep(std::time::Duration::from_millis(100));
    chan = chan.sel1().send('e').zero();
    chan.sel2().close();
}

fn main() {
    let (chan1, chan2) = session_channel();
    let join_handle = std::thread::spawn(move || upcase(chan1));
    chargen(chan2);
    join_handle.join().unwrap();
}
