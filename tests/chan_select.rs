#[macro_use] extern crate rust_sessions;

use std::thread::spawn;
use std::borrow::ToOwned;
use rust_sessions::*;

// recv and assert a value, then close the channel
macro_rules! recv_assert_eq_close(
    ($e:expr, $rx:ident.recv())
        =>
    ({
        let (c, v) = $rx.recv();
        assert_eq!($e, v);
        c.close();
    })
);

#[test]
fn chan_select_simple() {
    let (tcs, rcs) = session_channel();
    let (tcu, rcu) = session_channel();

    // Spawn threads
    send_str(tcs);

    // The lifetime of `sel` is reduced to the point where we call
    // `wait()`. This ensures we don't hold on to Chan references, but still
    // prevents using the channels the ChanSelect holds references to.
    let index = {
        let mut sel = ChanSelect::new();
        sel.add_recv(&rcs); // Assigned 0
        sel.add_recv(&rcu); // Assigned 1
        sel.wait()     // Destroys the ChanSelect, releases references to
            // rcs and rcu
    };

    assert_eq!(0, index);
    recv_assert_eq_close!("Hello, World!".to_owned(), rcs.recv());

    let (tcs, rcs) = session_channel();

    send_usize(tcu);

    let index = {
        let mut sel = ChanSelect::new();
        sel.add_recv(&rcs);
        sel.add_recv(&rcu);
        sel.wait()
    };

    assert_eq!(1, index);
    recv_assert_eq_close!(42, rcu.recv());

    // Not really necessary for the test, just used to coerce the types of
    // tcs and rcs
    send_str(tcs);
    recv_assert_eq_close!("Hello, World!".to_owned(), rcs.recv());
}

#[test]
fn chan_select_add_ret() {
    enum ChanToRead {
        Str,
        Usize
    }

    let (tcs, rcs) = session_channel();
    let (tcu, rcu) = session_channel();

    // Spawn threads
    spawn(move|| send_str(tcs));

    // The lifetime of `sel` is reduced to the point where we call
    // `wait()`. This ensures we don't hold on to Chan references, but still
    // prevents using the channels the ChanSelect holds references to.
    let chan_to_read = {
        let mut sel = ChanSelect::new();
        sel.add_recv_ret(&rcs, ChanToRead::Str);   // Assigned 0
        sel.add_recv_ret(&rcu, ChanToRead::Usize); // Assigned 1
        sel.wait()     // Destroys the ChanSelect, releases references to
            // rcs and rcu
    };

    send_usize(tcu);

    match chan_to_read {
        ChanToRead::Str => {
            recv_assert_eq_close!("Hello, World!".to_owned(), rcs.recv());
            recv_assert_eq_close!(42, rcu.recv());
        }
        ChanToRead::Usize => {
            panic!("Unexpected read of usize chan before str chan!");
        }
    }
}

// Utility functions

fn send_str(c: Chan<(), Send<String, Eps>>) {
    c.send("Hello, World!".to_string()).close();
}

fn send_usize(c: Chan<(), Send<usize, Eps>>) {
    c.send(42).close();
}
