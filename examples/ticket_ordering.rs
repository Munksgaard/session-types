#![feature(box_syntax)]

extern crate rust_sessions;
extern crate rand;

use rust_sessions::*;

use std::thread::spawn;
use std::sync::mpsc::channel;
use std::io::Write;

use rand::{thread_rng, Rand};

/*
 * This is the type we need to implement (from Hu-Yoshida-Honda):
 *
 * protocol placeOrder {
 *   begin.
 *   ![ !<String>.?(Double) ]*.     // ![]* construct is iterating construct, a kind of rec-choose
 *   !{ ACCEPT: !<Address>.?(Date), // !{} is branching construct (multi-way choose with labels)
 *      REJECT:
 *    }
 * }
 *
 */

type TravelAgency = IterOut<Recv<String, Send<f64, IterEps>>, Offer<Recv<String, Send<u64, Eps>>, Eps>>;
type Client = IterIn<Send<String, Recv<f64, IterEps>>, Choose<Send<String, Recv<u64, Eps>>, Eps>>;

macro_rules! query {
    ($q:expr) => ({
        print!($q);
        let _ = std::io::stdout().flush();
    })
}

fn cli(c: Chan<(), Client>) {

    let mut buf = "".to_string();
    let mut stdin = std::io::stdin();
    let mut loop_chan = c;
    let end;
    loop {
        query!("Another query? (Y/n) ");

        buf.clear();
        stdin.read_line(&mut buf).ok().unwrap();

        match &buf[..] {
            "n\n" | "N\n" | "n" | "N" => {
                end = loop_chan.exit_while();
                break
            }
            _ => {
                let c = loop_chan.in_while();
                buf.clear();
                query!("Query: ");

                stdin.read_line(&mut buf).ok().unwrap();
                let c = c.send(buf.clone());
                let (c, price) = c.recv();
                println!("That costs {}", price);

                loop_chan = c.iter();
            }
        }
    }

    query!("Accept? (Y/n) ");
    buf.clear();
    stdin.read_line(&mut buf).ok().unwrap();
    match &buf[..] {
        "n\n" | "N\n" => {
            println!("Bye");
            end.sel2().close();
        }
        _ => {
            query!("Delivery address: ");

            buf.clear();
            stdin.read_line(&mut buf).ok().unwrap();

            let (c, date) = end.sel1().send(buf).recv();
            println!("Dispatch date: {}", date);
            println!("Nice doing business with you, bye!");
            c.close();
        }
    }
}

fn srv(c: Chan<(), TravelAgency>) {

    let mut g = thread_rng();

    let end;
    let mut loop_chan: Chan<(), TravelAgency> = c;

    loop {
        let c = match loop_chan.out_while() {
            Ok(c) => c,
            Err(c) => { end = c; break }
        };
        let (c, _) = c.recv();
        loop_chan = c.send(Rand::rand(&mut g)).iter();
    }

    // Alternative formulation - note that this version doesn't work, since the
    // typing cannot infer the resulting loop is the same as above.
    // while let Some(c) = match loop_chan.out_while() {
    //     Ok(c) => Some(c),
    //     Err(c) => { end = c; None }
    // } {
    //     let (c, _) = c.recv();
    //     loop_chan = c.send(Rand::rand(&mut g)).iter();
    // }

    match end.offer() {
        Ok(c) => {
            // Receive address and send date
            let (c, _) = c.recv();
            c.send(Rand::rand(&mut g)).close();
        }
        Err(c) => {
            // Declined
            c.close();
        }
    }
}

fn main() {
    let (travel_agency, client) = session_channel();

    spawn(move || srv(travel_agency));
    cli(client);
}
