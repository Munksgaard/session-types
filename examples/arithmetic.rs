// This is an implementation of the extended arithmetic server from
// Vasconcelos-Gay-Ravara (2006) with some additional functionality

#[macro_use]
extern crate session_types;
use session_types::*;

use std::thread::spawn;

// Offers: Add, Negate, Sqrt, Eval
type Srv =
    Offer<(Eps,
           Recv<i64, Recv<i64, Send<i64, Var<Z>>>>,
           Recv<i64, Send<i64, Var<Z>>>,
           Recv<f64, Choose<(Send<f64, Var<Z>>, Var<Z>)>>,
           Recv<fn(i64) -> bool, Recv<i64, Send<bool, Var<Z>>>>)>;

fn server(c: Chan<(), Rec<Srv>>) {
    let mut c = c.enter();
    loop {
        use session_types::Branch5::*;
        c = match c.offer() {
            B1(c) => {
                c.close();
                return
            },
            B2(c) => {
                let (c, n) = c.recv();
                let (c, m) = c.recv();
                c.send(n + m).zero()
            },
            B3(c) => {
                let (c, n) = c.recv();
                c.send(-n).zero()
            },
            B4(c) => {
                let (c, x) = c.recv();
                if x >= 0.0 {
                    c.sel1().send(x.sqrt()).zero()
                } else {
                    c.sel2().zero()
                }
            },
            B5(c) => {
                let (c, f) = c.recv();
                let (c, n) = c.recv();
                c.send(f(n)).zero()
            }
        }
    }
}

// `add_client`, `neg_client` and `sqrt_client` are all pretty straightforward
// uses of session types, but they do showcase subtyping, recursion and how to
// work the types in general.

type AddCli<R, S, T> =
    Choose<(Eps,
            Send<i64, Send<i64, Recv<i64, Var<Z>>>>,
            R, S, T)>;

fn add_client<R, S, T>(c: Chan<(), Rec<AddCli<R, S, T>>>) {
    let (c, n) = c.enter().sel2().send(42).send(1).recv();
    println!("{}", n);
    c.zero().sel1().close()
}

type NegCli<R, S, T> =
    Choose<(Eps,
            R,
            Send<i64, Recv<i64, Var<Z>>>,
            S, T)>;

fn neg_client<R, S, T>(c: Chan<(), Rec<NegCli<R, S, T>>>) {
    let (c, n) = c.enter().sel3().send(42).recv();
    println!("{}", n);
    c.zero().sel1().close();
}

type SqrtCli<R, S, T> =
    Choose<(Eps,
            R,
            S,
            Send<f64, Offer<(Recv<f64, Var<Z>>, Var<Z>)>>,
            T)>;

fn sqrt_client<R, S, T>(c: Chan<(), Rec<SqrtCli<R, S, T>>>) {
    match c.enter().sel4().send(42.0).offer() {
        B1(c) => {
            let (c, n) = c.recv();
            println!("{}", n);
            c.zero().sel1().close();
        }
        B2(c) => {
            println!("Couldn't take square root!");
            c.zero().sel1().close();
        }
    }
}

// `fn_client` sends a function over the channel

type PrimeCli<R, S, T> =
    Choose<(Eps,
            R,
            S,
            T,
            Send<fn(i64) -> bool, Send<i64, Recv<bool, Var<Z>>>>)>;

fn fn_client<R, S, T>(c: Chan<(), Rec<PrimeCli<R, S, T>>>) {
    fn even(n: i64) -> bool {
        n % 2 == 0
    }

    let (c, b) = c.enter()
        .sel5()
        .send(even)
        .send(42)
        .recv();
    println!("{}", b);
    c.zero().sel1().close();
}

// `ask_neg` and `get_neg` use delegation, that is, sending a channel over
// another channel.

// `ask_neg` selects the negation operation and sends an integer, whereafter it
// sends the whole channel to `get_neg`. `get_neg` then receives the negated
// integer and prints it.

type AskNeg<R, S, T> =
    Choose<(Eps,
            R,
            Send<i64, Recv<i64, Var<Z>>>,
            S, T)>;


fn ask_neg<R: std::marker::Send + 'static, S: std::marker::Send + 'static, T: std::marker::Send + 'static>
    (c1: Chan<(), Rec<AskNeg<R, S, T>>>,
     c2: Chan<(), Send<Chan<(AskNeg<R, S, T>, ()), Recv<i64, Var<Z>>>, Eps>>) {
    let c1 = c1.enter().sel3().send(42);
    c2.send(c1).close();
}

fn get_neg<R: std::marker::Send + 'static, S: std::marker::Send + 'static, T: std::marker::Send + 'static>
    (c1: Chan<(), Recv<Chan<(AskNeg<R, S, T>, ()), Recv<i64, Var<Z>>>, Eps>>) {
    let (c1, c2) = c1.recv();
    let (c2, n) = c2.recv();
    println!("{}", n);
    c2.zero().sel1().close();
    c1.close();
}

fn main() {
    connect(server, add_client);
    connect(server, neg_client);
    connect(server, sqrt_client);
    connect(server, fn_client);

    let (c1, c1_) = session_channel();
    let (c2, c2_) = session_channel();

    let t1 = spawn(move || server(c1));
    let t2 = spawn(move || ask_neg(c1_, c2));
    let t3 = spawn(move || get_neg(c2_));

    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
}
