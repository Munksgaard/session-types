Session Types for Rust
----------------------

This is an implementation of [session types for Rust](http://munksgaard.me/papers/laumann-munksgaard-larsen.pdf).

Using this library, you can implement **bi-directional process communication**
with compile-time assurance that neither party will violate the communication
protocol.

[![Build Status](https://travis-ci.org/Munksgaard/session-types.svg?branch=master)](https://travis-ci.org/Munksgaard/session-types) [![Cargo](https://img.shields.io/crates/v/session_types.svg)](https://crates.io/crates/session_types) [![Documentation](https://docs.rs/session_types/badge.svg)](https://docs.rs/session_types)

## Getting started

[session-types is available on crates.io](https://crates.io/crates/session_types). It is recommended to look there for the newest released version, as well as links to the newest builds of the docs.

At the point of the last update of this README, the latest published version could be used like this:

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
session_types = "0.2.0"
```

...and see the [docs](https://docs.rs/session_types/) for how to use it.

## Example

```rust
extern crate session_types;
use session_types::*;
use std::thread;

type Server = Recv<i64, Send<bool, Eps>>;
type Client = <Server as HasDual>::Dual;

fn srv(c: Chan<(), Server>) {
    let (c, n) = c.recv();
    if n % 2 == 0 {
        c.send(true).close()
    } else {
        c.send(false).close()
    }
}

fn cli(c: Chan<(), Client>) {
    let n = 42;
    let c = c.send(n);
    let (c, b) = c.recv();

    if b {
        println!("{} is even", n);
    } else {
        println!("{} is odd", n);
    }

    c.close();
}

fn main() {
    let (server_chan, client_chan) = session_channel();

    let srv_t = thread::spawn(move || srv(server_chan));
    let cli_t = thread::spawn(move || cli(client_chan));

    let _ = (srv_t.join(), cli_t.join());
}
```

We start by specifying a _protocol_. Protocols are constructed from the protocol
types `Send`, `Recv`, `Choose`, `Offer`, `Rec` and `Var`. In this case, our
protocol is:

```rust
type Server = Recv<i64, Send<bool, Eps>>;
```

which reads:

 * Receive a signed 64-bit integer
 * Send a boolean
 * Close the channel

The `Client` protocol is the _dual_, which is a well-defined concept in session
types. Loosely, it means that every protocol step in one process has a matching
step in the other. If one process wants to send a value, the other process must
be ready to receive it. The dual of `Server` is:

```rust
type Client = Send<i64, Recv<bool, Eps>>;
```

With `session-types`, we do not have to construct the dual by hand, as we
leverage the trait system to model this concept with the `HasDual` trait. This
allows us to do:

```rust
type Client = <Server as HasDual>::Dual;
```

For further information, check out [Session Types for
Rust](http://munksgaard.me/papers/laumann-munksgaard-larsen.pdf) and the
examples directory.
