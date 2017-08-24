extern crate session_types;
use session_types::*;

use std::thread::spawn;

fn client(n: u64, c: Chan<(), Send<u64, Eps>>) {
    c.send(n).close()
}

#[test]
fn main() {
    let n = 42;
    let (c1, c2) = session_channel();

    let res = c2.try_recv();
    assert!(res.is_err());
    let c2 = res.err().unwrap();

    let client_thread = spawn(move || client(n, c1));
    client_thread.join().unwrap();

    let res = c2.try_recv();
    assert!(res.is_ok());
    let (c, n_) = res.ok().unwrap();
    assert_eq!(n, n_);

    c.close();
}
