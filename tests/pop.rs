extern crate session_types;
use session_types::*;

fn client(n: u64, mut c: Chan<(), Rec<Send<u64, Var<Z>>>>) {
    let mut c = c.enter();
    c = c.send(n).pop(); // type should remain the same, test will fail if it cannot compile
}

fn client2(n: u64, mut c: Chan<(), Rec<Send<u64, Rec<Send<u64, Var<S<Z>>>>>>>) {
    let mut c = c.enter();
    c = c.send(n).enter().send(n).pop(); // type should remain the same, test will fail if it cannot compile
}

#[test]
fn main() {}