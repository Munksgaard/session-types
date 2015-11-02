extern crate session_types;
use session_types::*;

use std::thread::spawn;

fn client<'run>(n: u64, c: Chan2<'run, (), Send<u64, Eps>>) -> Complete<'run, Send<u64, Eps>> {
    c.send(n).close()
}

#[test]
fn main() {
    let n = 42;
    let (c1, c2) = session_channel();
    spawn(move || client(n, c1));

    let (c, n_) = c2.recv();
    c.close();
    assert_eq!(n, n_);
}
