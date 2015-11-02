extern crate session_types;
use session_types::*;

type Messenger = Send<i64, Send<bool, Eps>>;

fn send_some_send_all<'run_some, 'run_all>(c1: Chan2<'run_some, (), Messenger>, c2: Chan2<'run_all, (), Messenger>) -> Complete<'run_some, Messenger> {
    return c2.send(1).send(true).close(); //~ ERROR
}

fn main() {
    let (tx1, _) = session_channel();
    let (tx2, _) = session_channel();
    send_some_send_all(tx1, tx2);
}
