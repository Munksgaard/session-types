extern crate session_types;
use session_types::*;

use std::thread::spawn;

enum Msg {
    U64(u64)
}

fn client(n: u64, c: Chan<(), Send<Msg, Eps>, Msg>) {
    let n = Msg::U64(n);
    c.send(n).close()
}

#[test]
fn main() {
    let n = 42;
    let (c1, c2) = session_channel();
    spawn(move || client(n, c1));

    let (c, Msg::U64(n_)) = c2.recv();
    c.close();
    assert_eq!(n, n_);
}
