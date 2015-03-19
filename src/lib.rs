#![feature(std_misc)]

use std::marker;
use std::thread::scoped;
use std::mem::transmute;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;
use std::sync::mpsc::Select;
use std::marker::{PhantomData, PhantomFn};

pub struct Chan<E, T> (Sender<Box<u8>>, Receiver<Box<u8>>, PhantomData<(E, T)>);

fn unsafe_write_chan<A: marker::Send + 'static, E, T>(&Chan(ref tx, _, _): &Chan<E, T>, x: A) {
    let tx: &Sender<Box<A>> = unsafe { transmute(tx) };
    tx.send(Box::new(x)).unwrap();
}

fn unsafe_read_chan<A: marker::Send + 'static, E, T>(&Chan(_, ref rx, _): &Chan<E, T>) -> A {
    let rx: &Receiver<Box<A>> = unsafe { transmute(rx) };
    *rx.recv().unwrap()
}

// Peano numbers needed for Rec and Var
#[allow(missing_copy_implementations)]
pub struct Z;

pub struct S<P> ( PhantomData<P> );

#[allow(missing_copy_implementations)]
pub struct Eps;

pub struct Recv<A,R> ( PhantomData<(A, R)> );

pub struct Send<A,R> ( PhantomData<(A, R)> );

pub struct Choose<R,S> ( PhantomData<(R, S)> );

pub struct Offer<R,S> ( PhantomData<(R, S)> );

pub struct OfferTrait<T> ( PhantomData<T> );

pub struct Rec<R> ( PhantomData<R> );

pub struct Var<V> ( PhantomData<V> );

pub unsafe trait Dual: PhantomFn<Self> {}

unsafe impl Dual for (Eps, Eps) {}

unsafe impl <A, T, U> Dual for (Send<A, T>, Recv<A, U>)
    where (T, U): Dual {}

unsafe impl <A, T, U> Dual for (Recv<A, T>, Send<A, U>)
    where (T, U): Dual {}

unsafe impl<R, S, Rn, Sn> Dual for (Choose<R, Rn>, Offer<S, Sn>)
    where (R, S): Dual, (Rn, Sn): Dual {}

unsafe impl<R, S, Rn, Sn> Dual for (Offer<R, Rn>, Choose<S, Sn>)
    where (R, S): Dual, (Rn, Sn): Dual {}

unsafe impl Dual for (Var<Z>, Var<Z>) {}

unsafe impl<N> Dual for (Var<S<N>>, Var<S<N>>)
    where (Var<N>, Var<N>): Dual {}

unsafe impl<T, U> Dual for (Rec<T>, Rec<U>)
    where (T, U): Dual {}

pub unsafe trait EnvDual: PhantomFn<Self> {}

unsafe impl EnvDual for ((), ()) {}

unsafe impl <R, R_, T, T_> EnvDual for ((R, T), (R_, T_))
    where (R, R_): Dual,
          (T, T_): EnvDual {}

impl<E> Chan<E, Eps> {
    pub fn close(self) {
        // Consume `c`
    }
}

impl<E, T, A: marker::Send + 'static> Chan<E, Send<A, T>> {
    pub fn send(self, v: A) -> Chan<E, T> {
        unsafe_write_chan(&self, v);
        unsafe { transmute(self) }
    }
}

impl<E, T, A: marker::Send + 'static> Chan<E, Recv<A, T>> {
    pub fn recv(self) -> (Chan<E, T>, A) {
        let v = unsafe_read_chan(&self);
        (unsafe { transmute(self) }, v)
    }
}

impl<E, R, S> Chan<E, Choose<R, S>> {
    pub fn sel1(self) -> Chan<E, R> {
        unsafe_write_chan(&self, true);
        unsafe { transmute(self) }
    }

    pub fn sel2(self) -> Chan<E, S> {
        unsafe_write_chan(&self, false);
        unsafe { transmute(self) }
    }
}

impl<Z, A, B> Chan<Z, Choose<A, B>> {
    pub fn skip(self) -> Chan<Z, B> {
        self.sel2()
    }
}

impl<Z, A, B, C> Chan<Z, Choose<A, Choose<B, C>>> {
    pub fn skip2(self) -> Chan<Z, C> {
        self.sel2().sel2()
    }
}

impl<Z, A, B, C, D> Chan<Z, Choose<A, Choose<B, Choose<C, D>>>> {
    pub fn skip3(self) -> Chan<Z, D> {
        self.sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, E>>>>> {
    pub fn skip4(self) -> Chan<Z, E> {
        self.sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, F>>>>>> {
    pub fn skip5(self) -> Chan<Z, F> {
        self.sel2().sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F, G> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, Choose<F, G>>>>>>> {
    pub fn skip6(self) -> Chan<Z, G> {
        self.sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F, G, H> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, Choose<E, Choose<F, Choose<G, H>>>>>>>> {
    pub fn skip7(self) -> Chan<Z, H> {
        self.sel2().sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

impl<E, R, S> Chan<E, Offer<R, S>> {
    pub fn offer(self) -> Result<Chan<E, R>, Chan<E, S>> {
        let b = unsafe_read_chan(&self);
        if b {
            Ok(unsafe { transmute(self) })
        } else {
            Err(unsafe { transmute(self) })
        }
    }
}

impl<E, R> Chan<E, Rec<R>> {
    pub fn enter(self) -> Chan<(R, E), R> {
        unsafe { transmute(self) }
    }
}

impl<E, R> Chan<(R, E), Var<Z>> {
    pub fn zero(self) -> Chan<(R, E), R> {
        unsafe { transmute(self) }
    }
}

impl<E, R, V> Chan<(R, E), Var<S<V>>> {
    pub fn succ(self) -> Chan<E, Var<V>> {
        unsafe { transmute(self) }
    }
}

// Homogeneous select. We have a list of channels, all obeying the same protocol
// (and in the exact same point of the protocol)
pub fn hselect<E, P, A>(mut chans: Vec<Chan<E, Recv<A, P>>>) -> (Chan<E, Recv<A, P>>, Vec<Chan<E, Recv<A, P>>>)
{
    let i = iselect(&chans);
    let c = chans.remove(i);
    (c, chans)
}

// An alternative version of homogeneous select, returning the index of the Chan
// that is ready to receive.
pub fn iselect<E, P, A>(chans: &Vec<Chan<E, Recv<A, P>>>) -> usize {
    let mut map = HashMap::new();

    let id = {
        let sel = Select::new();
        let mut handles = vec![]; // collect all the handles

        for (i, chan) in chans.iter().enumerate() {
            let &Chan(_, ref rx, _) = chan;
            let handle = sel.handle(rx);
            map.insert(handle.id(), i);
            handles.push(handle);
        }

        for handle in handles.iter_mut() { // Add
            unsafe { handle.add(); }
        }

        let id = sel.wait();

        for handle in handles.iter_mut() { // Clean up
            unsafe { handle.remove(); }
        }

        id
    };
    map.remove(&id).unwrap()
}

/// Heterogeneous selection structure for channels
///
/// This builds a structure of receiver that we wish to select over. This is a
/// little dangerous, because we transmute the inner Receiver to a 'static
/// lifetime borrow, and if the receiver goes out of scope calling `wait` will
/// most likely cause a crash.
///
pub struct ChanSelect<'c, T> {
    rxs: Vec<&'c Chan<(), ()>>,
    ret: Vec<T>
}

impl<'c, T> ChanSelect<'c, T> {
    pub fn new() -> ChanSelect<'c, T> {
        ChanSelect {
            rxs: Vec::new(),
            ret: Vec::new()
        }
    }

    /// Add a channel whose next step is Recv
    ///
    /// This method is marked unsafe, because of the lifetime transmute. If the
    /// Receiver goes out of scope nasty things can happen.
    pub fn add_ret<E, R, A: marker::Send>(&mut self,
                                               chan: &'c Chan<E, Recv<A, R>>,
                                               ret: T)
    {
        self.ret.push(ret);
        self.rxs.push(unsafe { transmute(chan) });
    }

    /// Find a Receiver (and hence a Chan) that is ready to receive.
    ///
    /// This method consumes the ChanSelect, freeing up the borrowed Receivers
    /// to be consumed.
    pub fn wait(mut self) -> T {
        let sel = Select::new();
        let mut handles = Vec::with_capacity(self.rxs.len());
        let mut map = HashMap::new();

        for (i, chan) in self.rxs.iter().enumerate() {
            let &Chan(_, ref rx, _) = *chan;
            let h = sel.handle(rx);
            let id = h.id();
            map.insert(id, i);
            handles.push(h);
        }

        for handle in handles.iter_mut() {
            unsafe { handle.add(); }
        }

        let id = sel.wait();

        for handle in handles.iter_mut() {
            unsafe { handle.remove(); }
        }
        let index = map.remove(&id).unwrap();
        self.ret.remove(index)
    }
}

impl<'c> ChanSelect<'c, usize> {
    pub fn add<E, R, A: marker::Send>(&mut self,
                                           c: &'c Chan<E, Recv<A, R>>)
    {
        let index = self.rxs.len();
        self.add_ret(c, index);
    }
}

pub fn accept<E: marker::Send + 'static, R: marker::Send + 'static>(tx: Sender<Chan<E, R>>) -> Option<Chan<E, R>> {
    borrow_accept(&tx)
}

pub fn borrow_accept<E: marker::Send + 'static, R: marker::Send + 'static>(tx: &Sender<Chan<E, R>>) -> Option<Chan<E, R>> {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let c1 = Chan (tx1, rx2, PhantomData);
    let c2 = Chan (tx2, rx1, PhantomData);

    match tx.send(c1) {
        Ok(_) => Some(c2),
        _ => None,
    }
}

pub fn request<E: marker::Send + 'static, E_, R: marker::Send + 'static, S>(rx: Receiver<Chan<E, R>>) -> Option<Chan<E_, S>>
    where (R, S): Dual,
          (E, E_): EnvDual
{
    borrow_request(&rx)
}

pub fn borrow_request<E: marker::Send + 'static, E_, R: marker::Send + 'static, S>(rx: &Receiver<Chan<E, R>>) -> Option<Chan<E_, S>>
    where (R, S): Dual,
          (E, E_): EnvDual
{
    match rx.recv() {
        Ok(c) => Some(unsafe { transmute(c) }),
        _ => None,
    }
}

pub fn session_channel<E: marker::Send + 'static, E_, R: marker::Send + 'static, S>() -> (Chan<E, R>, Chan<E_, S>)
    where (R, S): Dual,
          (E, E_): EnvDual
{
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let c1 = Chan(tx1, rx2, PhantomData);
    let c2 = Chan(tx2, rx1, PhantomData);

    (c1, c2)
}

pub fn connect<E: marker::Send + 'static, E_, F1, F2, R: marker::Send + 'static, S>(srv: F1, cli: F2)
    where F1: Fn(Chan<E, R>) + marker::Send,
          F2: Fn(Chan<E_, S>) + marker::Send,
          (R, S): Dual,
          (E, E_): EnvDual
{
    let (tx, rx) = channel();
    let joinguard = scoped(move || srv(accept(tx).unwrap()));
    cli(request(rx).unwrap());
    joinguard.join();
}



#[macro_export]
macro_rules! offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.offer() {
            Ok($id) => $code,
            Err($id) => offer!{ $id, $($t)+ }
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}

#[macro_export]
macro_rules! chan_select {
    (
        $(($c:ident, $name:pat) = $rx:ident.recv() => $code:expr),+
    ) => ({
        let index = {
            let mut sel = $crate::ChanSelect::new();
            $( sel.add(&$rx); )+
            sel.wait()
        };

        let mut i: usize = 0;

        $( if index == { i += 1; i - 1 } { let ($c, $name) = $rx.recv(); $code } else )+
        { unreachable!() }
    })
}
