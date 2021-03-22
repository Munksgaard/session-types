//! `session_types`
//!
//! This is an implementation of *session types* in Rust.
//!
//! The channels in Rusts standard library are useful for a great many things,
//! but they're restricted to a single type. Session types allows one to use a
//! single channel for transferring values of different types, depending on the
//! context in which it is used. Specifically, a session typed channel always
//! carry a *protocol*, which dictates how communication is to take place.
//!
//! For example, imagine that two threads, `A` and `B` want to communicate with
//! the following pattern:
//!
//!  1. `A` sends an integer to `B`.
//!  2. `B` sends a boolean to `A` depending on the integer received.
//!
//! With session types, this could be done by sharing a single channel. From
//! `A`'s point of view, it would have the type `int ! (bool ? eps)` where `t ! r`
//! is the protocol "send something of type `t` then proceed with
//! protocol `r`", the protocol `t ? r` is "receive something of type `t` then proceed
//! with protocol `r`, and `eps` is a special marker indicating the end of a
//! communication session.
//!
//! Our session type library allows the user to create channels that adhere to a
//! specified protocol. For example, a channel like the above would have the type
//! `Chan<(), Send<i64, Recv<bool, Eps>>>`, and the full program could look like this:
//!
//! ```
//! extern crate session_types;
//! use session_types::*;
//!
//! type Server = Recv<i64, Send<bool, Eps>>;
//! type Client = Send<i64, Recv<bool, Eps>>;
//!
//! fn srv(c: Chan<(), Server>) {
//!     let (c, n) = c.recv();
//!     if n % 2 == 0 {
//!         c.send(true).close()
//!     } else {
//!         c.send(false).close()
//!     }
//! }
//!
//! fn cli(c: Chan<(), Client>) {
//!     let n = 42;
//!     let c = c.send(n);
//!     let (c, b) = c.recv();
//!
//!     if b {
//!         println!("{} is even", n);
//!     } else {
//!         println!("{} is odd", n);
//!     }
//!
//!     c.close();
//! }
//!
//! fn main() {
//!     connect(srv, cli);
//! }
//! ```
#![cfg_attr(feature = "cargo-clippy", allow(clippy::double_must_use))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
extern crate crossbeam_channel;

use std::marker::PhantomData;
use std::thread::spawn;
use std::{marker, mem, ptr};

use std::collections::HashMap;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crossbeam_channel::Select;

pub use Branch::*;

/// A session typed channel. `P` is the protocol and `E` is the environment,
/// containing potential recursion targets
#[must_use]
pub struct Chan<E, P>(Sender<*mut u8>, Receiver<*mut u8>, PhantomData<(E, P)>);

unsafe impl<E: marker::Send, P: marker::Send> marker::Send for Chan<E, P> {}

unsafe fn write_chan<A: marker::Send + 'static, E, P>(&Chan(ref tx, _, _): &Chan<E, P>, x: A) {
    tx.send(Box::into_raw(Box::new(x)) as *mut _).unwrap()
}

unsafe fn read_chan<A: marker::Send + 'static, E, P>(&Chan(_, ref rx, _): &Chan<E, P>) -> A {
    *Box::from_raw(rx.recv().unwrap() as *mut A)
}

unsafe fn try_read_chan<A: marker::Send + 'static, E, P>(
    &Chan(_, ref rx, _): &Chan<E, P>,
) -> Option<A> {
    match rx.try_recv() {
        Ok(a) => Some(*Box::from_raw(a as *mut A)),
        Err(_) => None,
    }
}

/// Peano numbers: Zero
#[allow(missing_copy_implementations)]
pub struct Z;

/// Peano numbers: Increment
pub struct S<N>(PhantomData<N>);

/// End of communication session (epsilon)
#[allow(missing_copy_implementations)]
pub struct Eps;

/// Receive `A`, then `P`
pub struct Recv<A, P>(PhantomData<(A, P)>);

/// Send `A`, then `P`
pub struct Send<A, P>(PhantomData<(A, P)>);

/// Active choice between `P` and `Q`
pub struct Choose<P, Q>(PhantomData<(P, Q)>);

/// Passive choice (offer) between `P` and `Q`
pub struct Offer<P, Q>(PhantomData<(P, Q)>);

/// Enter a recursive environment
pub struct Rec<P>(PhantomData<P>);

/// Recurse. N indicates how many layers of the recursive environment we recurse
/// out of.
pub struct Var<N>(PhantomData<N>);

/// The HasDual trait defines the dual relationship between protocols.
///
/// Any valid protocol has a corresponding dual.
///
/// This trait is sealed and cannot be implemented outside of session-types
pub trait HasDual: private::Sealed {
    type Dual;
}

impl HasDual for Eps {
    type Dual = Eps;
}

impl<A, P: HasDual> HasDual for Send<A, P> {
    type Dual = Recv<A, P::Dual>;
}

impl<A, P: HasDual> HasDual for Recv<A, P> {
    type Dual = Send<A, P::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Choose<P, Q> {
    type Dual = Offer<P::Dual, Q::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Offer<P, Q> {
    type Dual = Choose<P::Dual, Q::Dual>;
}

impl HasDual for Var<Z> {
    type Dual = Var<Z>;
}

impl<N> HasDual for Var<S<N>> {
    type Dual = Var<S<N>>;
}

impl<P: HasDual> HasDual for Rec<P> {
    type Dual = Rec<P::Dual>;
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

impl<E, P> Drop for Chan<E, P> {
    fn drop(&mut self) {
        panic!("Session channel prematurely dropped");
    }
}

impl<E> Chan<E, Eps> {
    /// Close a channel. Should always be used at the end of your program.
    pub fn close(self) {
        // This method cleans up the channel without running the panicky destructor
        // In essence, it calls the drop glue bypassing the `Drop::drop` method

        let this = mem::ManuallyDrop::new(self);

        let sender = unsafe { ptr::read(&(this).0 as *const _) };
        let receiver = unsafe { ptr::read(&(this).1 as *const _) };

        drop(sender);
        drop(receiver); // drop them
    }
}

impl<E, P> Chan<E, P> {
    unsafe fn cast<E2, P2>(self) -> Chan<E2, P2> {
        let this = mem::ManuallyDrop::new(self);
        Chan(
            ptr::read(&(this).0 as *const _),
            ptr::read(&(this).1 as *const _),
            PhantomData,
        )
    }
}

impl<E, P, A: marker::Send + 'static> Chan<E, Send<A, P>> {
    /// Send a value of type `A` over the channel. Returns a channel with
    /// protocol `P`
    #[must_use]
    pub fn send(self, v: A) -> Chan<E, P> {
        unsafe {
            write_chan(&self, v);
            self.cast()
        }
    }
}

impl<E, P, A: marker::Send + 'static> Chan<E, Recv<A, P>> {
    /// Receives a value of type `A` from the channel. Returns a tuple
    /// containing the resulting channel and the received value.
    #[must_use]
    pub fn recv(self) -> (Chan<E, P>, A) {
        unsafe {
            let v = read_chan(&self);
            (self.cast(), v)
        }
    }

    /// Non-blocking receive.
    #[must_use]
    pub fn try_recv(self) -> Result<(Chan<E, P>, A), Self> {
        unsafe {
            if let Some(v) = try_read_chan(&self) {
                Ok((self.cast(), v))
            } else {
                Err(self)
            }
        }
    }
}

impl<E, P, Q> Chan<E, Choose<P, Q>> {
    /// Perform an active choice, selecting protocol `P`.
    #[must_use]
    pub fn sel1(self) -> Chan<E, P> {
        unsafe {
            write_chan(&self, true);
            self.cast()
        }
    }

    /// Perform an active choice, selecting protocol `Q`.
    #[must_use]
    pub fn sel2(self) -> Chan<E, Q> {
        unsafe {
            write_chan(&self, false);
            self.cast()
        }
    }
}

/// Convenience function. This is identical to `.sel2()`
impl<Z, A, B> Chan<Z, Choose<A, B>> {
    #[must_use]
    pub fn skip(self) -> Chan<Z, B> {
        self.sel2()
    }
}

/// Convenience function. This is identical to `.sel2().sel2()`
impl<Z, A, B, C> Chan<Z, Choose<A, Choose<B, C>>> {
    #[must_use]
    pub fn skip2(self) -> Chan<Z, C> {
        self.sel2().sel2()
    }
}

/// Convenience function. This is identical to `.sel2().sel2().sel2()`
impl<Z, A, B, C, D> Chan<Z, Choose<A, Choose<B, Choose<C, D>>>> {
    #[must_use]
    pub fn skip3(self) -> Chan<Z, D> {
        self.sel2().sel2().sel2()
    }
}

/// Convenience function. This is identical to `.sel2().sel2().sel2().sel2()`
impl<Z, A, B, C, D, E> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, E>>>>> {
    #[must_use]
    pub fn skip4(self) -> Chan<Z, E> {
        self.sel2().sel2().sel2().sel2()
    }
}

/// Convenience function. This is identical to `.sel2().sel2().sel2().sel2().sel2()`
impl<Z, A, B, C, D, E, F> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, F>>>>>> {
    #[must_use]
    pub fn skip5(self) -> Chan<Z, F> {
        self.sel2().sel2().sel2().sel2().sel2()
    }
}

/// Convenience function.
impl<Z, A, B, C, D, E, F, G>
    Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, Choose<F, G>>>>>>>
{
    #[must_use]
    pub fn skip6(self) -> Chan<Z, G> {
        self.sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

/// Convenience function.
impl<Z, A, B, C, D, E, F, G, H>
    Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, Choose<F, Choose<G, H>>>>>>>>
{
    #[must_use]
    pub fn skip7(self) -> Chan<Z, H> {
        self.sel2().sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

impl<E, P, Q> Chan<E, Offer<P, Q>> {
    /// Passive choice. This allows the other end of the channel to select one
    /// of two options for continuing the protocol: either `P` or `Q`.
    #[must_use]
    pub fn offer(self) -> Branch<Chan<E, P>, Chan<E, Q>> {
        unsafe {
            let b = read_chan(&self);
            if b {
                Left(self.cast())
            } else {
                Right(self.cast())
            }
        }
    }

    /// Poll for choice.
    #[must_use]
    pub fn try_offer(self) -> Result<Branch<Chan<E, P>, Chan<E, Q>>, Self> {
        unsafe {
            if let Some(b) = try_read_chan(&self) {
                if b {
                    Ok(Left(self.cast()))
                } else {
                    Ok(Right(self.cast()))
                }
            } else {
                Err(self)
            }
        }
    }
}

impl<E, P> Chan<E, Rec<P>> {
    /// Enter a recursive environment, putting the current environment on the
    /// top of the environment stack.
    #[must_use]
    pub fn enter(self) -> Chan<(P, E), P> {
        unsafe { self.cast() }
    }
}

impl<E, P> Chan<(P, E), Var<Z>> {
    /// Recurse to the environment on the top of the environment stack.
    #[must_use]
    pub fn zero(self) -> Chan<(P, E), P> {
        unsafe { self.cast() }
    }
}

impl<E, P, N> Chan<(P, E), Var<S<N>>> {
    /// Pop the top environment from the environment stack.
    #[must_use]
    pub fn succ(self) -> Chan<E, Var<N>> {
        unsafe { self.cast() }
    }
}

/// Homogeneous select. We have a vector of channels, all obeying the same
/// protocol (and in the exact same point of the protocol), wait for one of them
/// to receive. Removes the receiving channel from the vector and returns both
/// the channel and the new vector.
#[must_use]
pub fn hselect<E, P, A>(
    mut chans: Vec<Chan<E, Recv<A, P>>>,
) -> (Chan<E, Recv<A, P>>, Vec<Chan<E, Recv<A, P>>>) {
    let i = iselect(&chans);
    let c = chans.remove(i);
    (c, chans)
}

/// An alternative version of homogeneous select, returning the index of the Chan
/// that is ready to receive.
pub fn iselect<E, P, A>(chans: &[Chan<E, Recv<A, P>>]) -> usize {
    let mut map = HashMap::new();

    let id = {
        let mut sel = Select::new();
        let mut handles = Vec::with_capacity(chans.len()); // collect all the handles

        for (i, chan) in chans.iter().enumerate() {
            let &Chan(_, ref rx, _) = chan;
            let handle = sel.recv(rx);
            map.insert(handle, i);
            handles.push(handle);
        }

        sel.ready()
    };
    map.remove(&id).unwrap()
}

/// Heterogeneous selection structure for channels
///
/// This builds a structure of channels that we wish to select over. This is
/// structured in a way such that the channels selected over cannot be
/// interacted with (consumed) as long as the borrowing ChanSelect object
/// exists. This is necessary to ensure memory safety.
///
/// The type parameter T is a return type, ie we store a value of some type T
/// that is returned in case its associated channels is selected on `wait()`
pub struct ChanSelect<'c> {
    receivers: Vec<&'c Receiver<*mut u8>>,
}

impl<'c> ChanSelect<'c> {
    pub fn new() -> ChanSelect<'c> {
        ChanSelect {
            receivers: Vec::new(),
        }
    }

    /// Add a channel whose next step is `Recv`
    ///
    /// Once a channel has been added it cannot be interacted with as long as it
    /// is borrowed here (by virtue of borrow checking and lifetimes).
    pub fn add_recv<E, P, A: marker::Send>(&mut self, chan: &'c Chan<E, Recv<A, P>>) {
        let &Chan(_, ref rx, _) = chan;
        let _ = self.receivers.push(rx);
    }

    pub fn add_offer<E, P, Q>(&mut self, chan: &'c Chan<E, Offer<P, Q>>) {
        let &Chan(_, ref rx, _) = chan;
        let _ = self.receivers.push(rx);
    }

    /// Find a Receiver (and hence a Chan) that is ready to receive.
    ///
    /// This method consumes the ChanSelect, freeing up the borrowed Receivers
    /// to be consumed.
    pub fn wait(self) -> usize {
        let mut sel = Select::new();
        for rx in self.receivers.into_iter() {
            sel.recv(rx);
        }

        sel.ready()
    }

    /// How many channels are there in the structure?
    pub fn len(&self) -> usize {
        self.receivers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receivers.is_empty()
    }
}

impl<'c> Default for ChanSelect<'c> {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns two session channels
#[must_use]
pub fn session_channel<P: HasDual>() -> (Chan<(), P>, Chan<(), P::Dual>) {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();

    let c1 = Chan(tx1, rx2, PhantomData);
    let c2 = Chan(tx2, rx1, PhantomData);

    (c1, c2)
}

/// Connect two functions using a session typed channel.
pub fn connect<F1, F2, P>(srv: F1, cli: F2)
where
    F1: Fn(Chan<(), P>) + marker::Send + 'static,
    F2: Fn(Chan<(), P::Dual>) + marker::Send,
    P: HasDual + marker::Send + 'static,
    P::Dual: HasDual + marker::Send + 'static,
{
    let (c1, c2) = session_channel();
    let t = spawn(move || srv(c1));
    cli(c2);
    t.join().unwrap();
}

mod private {
    use super::*;
    pub trait Sealed {}

    // Impl for all exported protocol types
    impl Sealed for Eps {}
    impl<A, P> Sealed for Send<A, P> {}
    impl<A, P> Sealed for Recv<A, P> {}
    impl<P, Q> Sealed for Choose<P, Q> {}
    impl<P, Q> Sealed for Offer<P, Q> {}
    impl<Z> Sealed for Var<Z> {}
    impl<P> Sealed for Rec<P> {}
}

/// This macro is convenient for server-like protocols of the form:
///
/// `Offer<A, Offer<B, Offer<C, ... >>>`
///
/// # Examples
///
/// Assume we have a protocol `Offer<Recv<u64, Eps>, Offer<Recv<String, Eps>,Eps>>>`
/// we can use the `offer!` macro as follows:
///
/// ```rust
/// extern crate session_types;
/// use session_types::*;
/// use std::thread::spawn;
///
/// fn srv(c: Chan<(), Offer<Recv<u64, Eps>, Offer<Recv<String, Eps>, Eps>>>) {
///     offer! { c,
///         Number => {
///             let (c, n) = c.recv();
///             assert_eq!(42, n);
///             c.close();
///         },
///         String => {
///             c.recv().0.close();
///         },
///         Quit => {
///             c.close();
///         }
///     }
/// }
///
/// fn cli(c: Chan<(), Choose<Send<u64, Eps>, Choose<Send<String, Eps>, Eps>>>) {
///     c.sel1().send(42).close();
/// }
///
/// fn main() {
///     let (s, c) = session_channel();
///     spawn(move|| cli(c));
///     srv(s);
/// }
/// ```
///
/// The identifiers on the left-hand side of the arrows have no semantic
/// meaning, they only provide a meaningful name for the reader.
#[macro_export]
macro_rules! offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.offer() {
            $crate::Left($id) => $code,
            $crate::Right($id) => offer!{ $id, $($t)+ }
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}

/// Returns the channel unchanged on `TryRecvError::Empty`.
#[macro_export]
macro_rules! try_offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.try_offer() {
            Ok($crate::Left($id)) => $code,
            Ok($crate::Right($id)) => try_offer!{ $id, $($t)+ },
            Err($id) => Err($id)
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}

/// This macro plays the same role as the `select!` macro does for `Receiver`s.
///
/// It also supports a second form with `Offer`s (see the example below).
///
/// # Examples
///
/// ```rust
/// extern crate session_types;
/// use session_types::*;
/// use std::thread::spawn;
///
/// fn send_str(c: Chan<(), Send<String, Eps>>) {
///     c.send("Hello, World!".to_string()).close();
/// }
///
/// fn send_usize(c: Chan<(), Send<usize, Eps>>) {
///     c.send(42).close();
/// }
///
/// fn main() {
///     let (tcs, rcs) = session_channel();
///     let (tcu, rcu) = session_channel();
///
///     // Spawn threads
///     spawn(move|| send_str(tcs));
///     spawn(move|| send_usize(tcu));
///
///     chan_select! {
///         (c, s) = rcs.recv() => {
///             assert_eq!("Hello, World!".to_string(), s);
///             c.close();
///             rcu.recv().0.close();
///         },
///         (c, i) = rcu.recv() => {
///             assert_eq!(42, i);
///             c.close();
///             rcs.recv().0.close();
///         }
///     }
/// }
/// ```
///
/// ```rust
/// extern crate session_types;
/// extern crate rand;
///
/// use std::thread::spawn;
/// use session_types::*;
///
/// type Igo = Choose<Send<String, Eps>, Send<u64, Eps>>;
/// type Ugo = Offer<Recv<String, Eps>, Recv<u64, Eps>>;
///
/// fn srv(chan_one: Chan<(), Ugo>, chan_two: Chan<(), Ugo>) {
///     let _ign;
///     chan_select! {
///         _ign = chan_one.offer() => {
///             String => {
///                 let (c, s) = chan_one.recv();
///                 assert_eq!("Hello, World!".to_string(), s);
///                 c.close();
///                 match chan_two.offer() {
///                     Left(c) => c.recv().0.close(),
///                     Right(c) => c.recv().0.close(),
///                 }
///             },
///             Number => {
///                 chan_one.recv().0.close();
///                 match chan_two.offer() {
///                     Left(c) => c.recv().0.close(),
///                     Right(c) => c.recv().0.close(),
///                 }
///             }
///         },
///         _ign = chan_two.offer() => {
///             String => {
///                 chan_two.recv().0.close();
///                 match chan_one.offer() {
///                     Left(c) => c.recv().0.close(),
///                     Right(c) => c.recv().0.close(),
///                 }
///             },
///             Number => {
///                 chan_two.recv().0.close();
///                 match chan_one.offer() {
///                     Left(c) => c.recv().0.close(),
///                     Right(c) => c.recv().0.close(),
///                 }
///             }
///         }
///     }
/// }
///
/// fn cli(c: Chan<(), Igo>) {
///     c.sel1().send("Hello, World!".to_string()).close();
/// }
///
/// fn main() {
///     let (ca1, ca2) = session_channel();
///     let (cb1, cb2) = session_channel();
///
///     cb2.sel2().send(42).close();
///
///     spawn(move|| cli(ca2));
///
///     srv(ca1, cb1);
/// }
/// ```
#[macro_export]
macro_rules! chan_select {
    (
        $(($c:ident, $name:pat) = $rx:ident.recv() => $code:expr),+
    ) => ({
        let index = {
            let mut sel = $crate::ChanSelect::new();
            $( sel.add_recv(&$rx); )+
            sel.wait()
        };
        let mut i = 0;
        $( if index == { i += 1; i - 1 } { let ($c, $name) = $rx.recv(); $code }
           else )+
        { unreachable!() }
    });

    (
        $($res:ident = $rx:ident.offer() => { $($t:tt)+ }),+
    ) => ({
        let index = {
            let mut sel = $crate::ChanSelect::new();
            $( sel.add_offer(&$rx); )+
            sel.wait()
        };
        let mut i = 0;
        $( if index == { i += 1; i - 1 } { $res = offer!{ $rx, $($t)+ } } else )+
        { unreachable!() }
    })
}
