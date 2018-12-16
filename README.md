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

We start by specifying a _protocol_. Protocols are constructed from the
protocol types `Send`, `Recv`, `Choose`, `Offer`, `Rec` and `Var` (see
[Protocol types](#protocol-types)). In this case, our protocol is:

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

## Why is it cool?

Session types are not a new concept. They are cool, because they allow for
_compile-time_ verification of process communication. In other words, we are
able to check statically if two communicating processes adhere to their shared
communication protocol.

But implementing session types requires a way to model that certain actions in
the protocol have taken place. This is a complicated thing to do statically in
most programming languages, but in Rust it is easy because of _move semantics_.

Using move semantics, we ensure that "taking a step" in the protocol (sending,
receiving, etc) cannot be repeated.

## Protocol types

Any session-typed communication protocol is constructed from some basic
building blocks. This section goes through them pair-wise, showing an action
and its dual.

A session-typed channel is defined as `Chan<E, P>` where `E` is an
_environment_ and `P` is a protocol. The environment `E` is always `()` for
newly created channels (ie it is empty).

### `Eps`

This is the final step of any terminating protocol. The simplest example is:

    type Close = Eps;

Any channel whose type is `Chan<E, Eps>` implements the `close()` function that
closes the connection.

### `Send` and `Recv`

These are the most basic of actions: Transmitting and receiving values. The
opposite of a `Send` with some type `T` is a `Recv` of type `T`.

    type S = Send<String, Eps>;
    type R = Recv<String, Eps>; // <S as HasDual>::Dual

A channel of type `Chan<E, Send<T, P>>` implements the function `send(T) → Chan<E, P>`.

A channel of type `Chan<E, Recv<T, Q>>` implements the function `recv() → (T, Chan<E, Q>)`.

### `Choose` and `Offer`

There is the option of making choices in the protocol, for one process to
inform the other of a decision. The `Choose` and `Offer` constructs model such
choices.

    type C = Choose<Send<String, Eps>, Recv<String, Eps>>;
    type O = Offer<Recv<String, Eps>, Send<String, Eps>; // <C as Hasdual>::Dual

A channel of type `Chan<E, Choose<P, Q>>` implements _two_ functions:

 * `sel1() → Chan<E, P>`
 * `sel2() → Chan<E, Q>`

that communicates the choice of protocols `P` and `Q` to the other process.

A channel of type `Chan<E, Offer<P, Q>>` implements `offer() → Branch<Chan<E,
P>, Chan<E, Q>>`. `Branch` is an enum defined as:

    enum Branch<L, R> {
        Left(L),
        Right(R),
    }

A call to `offer()` should then be matched on to figure out which path the
other process decided to take.

    match c.offer() {
        Branch::Left(c) => …,
        Branch::Right(c) => …,
    }

### `Rec`, `Var`, `S` and `Z`

The type `Rec` implements the ability the recurse in the protocol, ie provides
an iterator component. The type `Rec<P>` allows repeating the protocol `P`.

A channel of type `Chan<E, Rec<P>>` implements the function `enter() → Chan<(P,
E), P>`. What `enter()` does is "store" the protocol `P` in the channel
environment.

The `Var` construct is then used to reference protocols stored in the
environment. As the environment is essentially a stack, `Var` takes a counter
that is modeled as a Peano number. `Var<Z>` points to the top of the stack.

A channel of type `Chan<(P, E), Var<Z>>` implements the function `zero() →
Chan<(P, E), P>`, ie `Var<Z>` is replaced by `P` at the top of the stack.

    type RS = Rec<Send<String, Var<Z>>>;
    type RR = Rec<Recv<String, Var<Z>>>; // <RS as HasDual>::Dual

The following program indefinitely sends some string:

    let c: Chan<(), Rec<Send<String, Var<Z>>> = …;
    let c = c.enter();
    loop {
        c = c.send("Hello!".to_string()).zero();
    }

Protocols in the environment can also be popped from the stack with
`Var<S<N>>`. This requires there to be at least one protocol in the stack.

A channel of type `Chan<(P, E), Var<S<N>>>` implements the function `succ() ->
Chan<E, Var<N>>` that peels away the `P` from the environment and `S` in the
counter.

With all the above, the following protocol should make sense:

    type Server = Rec<
                      Offer<
                          Eps,
                          Recv<String, Send<usize, Var<Z>>>
                      >
                  >;

It reads:

 * Either close the connection, or receive a string
 * If receive is chosen
   - Send back a `usize`
   - Go back to the beginning (from `Offer<…>`)

An example implementation:

    let c: Chan<(), Server> = …;
    let c = c.enter();
    loop {
        c = match c.offer() {
            Branch::Left(c) => {
                c.close();
                break
            },
            Branch::Right(c) => {
                let (c, str) = c.recv();
                c.send(str.len()).zero()
            }
        };
    }

## Additional reading and examples

For further information, check out [Session Types for
Rust](http://munksgaard.me/papers/laumann-munksgaard-larsen.pdf) and the
examples directory.
