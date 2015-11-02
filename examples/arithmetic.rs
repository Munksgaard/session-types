// This is an implementation of the extended arithmetic server from
// Vasconcelos-Gay-Ravara (2006) with some additional functionality

#[macro_use]
extern crate session_types;
use session_types::*;

use std::thread::spawn;

// Offers: Add, Negate, Sqrt, Eval
type Srv =
    Offer<Eps,
    Offer<Recv<i64, Recv<i64, Send<i64, Var<Z>>>>,
    Offer<Recv<i64, Send<i64, Var<Z>>>,
    Offer<Recv<f64, Choose<Send<f64, Var<Z>>, Var<Z>>>,
    Recv<fn(i64) -> bool, Recv<i64, Send<bool, Var<Z>>>>>>>>;

fn server<'run, TopP>(c: Chan<'run, TopP, (), Rec<Srv>>) -> Complete<'run, TopP> {
    let mut c = c.enter();
    loop {
        c = offer!{ c,
            CLOSE => {
                return c.close();
            },
            ADD => {
                let (c, n) = c.recv();
                let (c, m) = c.recv();
                c.send(n + m).zero()
            },
            NEGATE => {
                let (c, n) = c.recv();
                c.send(-n).zero()
            },
            SQRT => {
                let (c, x) = c.recv();
                if x >= 0.0 {
                    c.sel1().send(x.sqrt()).zero()
                } else {
                    c.sel2().zero()
                }
            },
            EVAL => {
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

type AddCli<R> =
    Choose<Eps,
    Choose<Send<i64, Send<i64, Recv<i64, Var<Z>>>>, R>>;

fn add_client<'run, R>(c: Chan2<'run, (), Rec<AddCli<R>>>) -> Complete<'run, Rec<AddCli<R>>> {
    let (c, n) = c.enter().sel2().sel1().send(42).send(1).recv();
    println!("{}", n);
    return c.zero().sel1().close()
}

type NegCli<R, S> =
    Choose<Eps,
    Choose<R,
    Choose<Send<i64, Recv<i64, Var<Z>>>,
    S>>>;

fn neg_client<'run, R, S>(c: Chan2<'run, (), Rec<NegCli<R, S>>>) -> Complete<'run, Rec<NegCli<R, S>>> {
    let (c, n) = c.enter().skip2().sel1().send(42).recv();
    println!("{}", n);
    return c.zero().sel1().close();
}

type SqrtCli<R, S, T> =
    Choose<Eps,
    Choose<R,
    Choose<S,
    Choose<Send<f64, Offer<Recv<f64, Var<Z>>, Var<Z>>>,
    T>>>>;

fn sqrt_client<'run, R, S, T>(c: Chan2<'run, (), Rec<SqrtCli<R, S, T>>>) -> Complete<'run, Rec<SqrtCli<R, S, T>>> {
    match c.enter().skip3().sel1().send(42.0).offer() {
        Left(c) => {
            let (c, n) = c.recv();
            println!("{}", n);
            return c.zero().sel1().close();
        }
        Right(c) => {
            println!("Couldn't take square root!");
            return c.zero().sel1().close();
        }
    }
}

// `fn_client` sends a function over the channel

type PrimeCli<R, S, T> =
    Choose<Eps,
    Choose<R,
    Choose<S,
    Choose<T,
    Send<fn(i64) -> bool, Send<i64, Recv<bool, Var<Z>>>>>>>>;

fn fn_client<'run, R, S, T>(c: Chan2<'run, (), Rec<PrimeCli<R, S, T>>>) -> Complete<'run, Rec<PrimeCli<R, S, T>>> {
    fn even(n: i64) -> bool {
        n % 2 == 0
    }

    let (c, b) = c.enter()
        .skip4()
        .send(even)
        .send(42)
        .recv();
    println!("{}", b);
    return c.zero().sel1().close();
}


// `ask_neg` and `get_neg` use delegation, that is, sending a channel over
// another channel.

// `ask_neg` selects the negation operation and sends an integer, whereafter it
// sends the whole channel to `get_neg`. `get_neg` then receives the negated
// integer and prints it.

type AskNeg<R, S> =
    Choose<Eps,
    Choose<R,
    Choose<Send<i64, Recv<i64, Var<Z>>>,
    S>>>;


fn ask_neg<R: std::marker::Send + 'static, S: std::marker::Send + 'static>(c1: ChanS<(), Rec<AskNeg<R, S>>>,
                 c2: ChanS<(), Send<Chan<'static, Rec<AskNeg<R, S>>, (AskNeg<R, S>, ()), Recv<i64, Var<Z>>>, Eps>>) -> Complete<'static, Send<Chan<'static, Rec<AskNeg<R, S>>, (AskNeg<R, S>, ()), Recv<i64, Var<Z>>>, Eps>>
{
    let c1 = c1.enter().sel2().sel2().sel1().send(42);
    return c2.send(c1).close();
}

fn get_neg<'run, TopP, R: std::marker::Send + 'static, S: std::marker::Send + 'static>(
   c: Chan<'run,
           TopP,
           (),
           Recv<Chan<'static, Rec<AskNeg<R, S>>, (AskNeg<R, S>, ()), Recv<i64, Var<Z>>>,
           Eps>>) -> Complete<'run, TopP> {
    let (c, cget) = c.recv();
    let (cget, n) = cget.recv();
    println!("{}", n);
    let proof: Complete<'static, Rec<AskNeg<R, S>>> = cget.zero().sel1().close();
    return c.close();
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
