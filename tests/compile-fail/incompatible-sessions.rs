extern crate session_types;

use std::thread::spawn;

use session_types::*;

fn srv(c: Chan<(), Recv<u8, Eps>>) {
    let (c, x) = c.recv();
    c.close();
}

fn main() {
    let (c1, c2) = session_channel();
    let t1 = spawn(|| { srv(c1) });

    srv(c2);                    //~ ERROR
    c2.close();                 //~ ERROR
    c2.sel1();                  //~ ERROR
    c2.sel2();                  //~ ERROR
    c2.offer();                 //~ ERROR
    c2.enter();                 //~ ERROR
    c2.zero();                  //~ ERROR
    c2.succ();                  //~ ERROR
    c2.recv();                  //~ ERROR

    c2.send(42).close();

    t1.join().unwrap();
}
