#![cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
extern crate session_types;
use session_types::*;
use std::thread::spawn;

type Id = String;
type Atm = Recv<Id, Choose<Rec<AtmInner>, Eps>>;

type AtmInner = Offer<AtmDeposit, Offer<AtmWithdraw, Offer<AtmBalance, Eps>>>;

type AtmDeposit = Recv<u64, Send<u64, Var<Z>>>;
type AtmWithdraw = Recv<u64, Choose<Var<Z>, Var<Z>>>;
type AtmBalance = Send<u64, Var<Z>>;

type Client = <Atm as HasDual>::Dual;

fn approved(id: &Id) -> bool {
    !id.is_empty()
}

fn atm(c: Chan<(), Atm>) {
    let mut c = {
        let (c, id) = c.recv();
        if !approved(&id) {
            c.sel2().close();
            return;
        }
        c.sel1().enter()
    };
    let mut balance = 0;
    loop {
        c = offer! {
            c,
            Deposit => {
                let (c, amt) = c.recv();
                balance += amt;
                c.send(balance).zero()
            },
            Withdraw => {
                let (c, amt) = c.recv();
                if amt > balance {
                    c.sel2().zero()
                } else {
                    balance -= amt;
                    c.sel1().zero()
                }
            },
            Balance => {
                c.send(balance).zero()
            },
            Quit => {
                c.close();
                break
            }
        }
    }
}

fn deposit_client(c: Chan<(), Client>) {
    let c = match c.send("Deposit Client".to_string()).offer() {
        Left(c) => c.enter(),
        Right(_) => panic!("deposit_client: expected to be approved"),
    };

    let (c, new_balance) = c.sel1().send(200).recv();
    println!("deposit_client: new balance: {}", new_balance);
    c.zero().skip3().close();
}

fn withdraw_client(c: Chan<(), Client>) {
    let c = match c.send("Withdraw Client".to_string()).offer() {
        Left(c) => c.enter(),
        Right(_) => panic!("withdraw_client: expected to be approved"),
    };

    match c.sel2().sel1().send(100).offer() {
        Left(c) => {
            println!("withdraw_client: Successfully withdrew 100");
            c.zero().skip3().close();
        }
        Right(c) => {
            println!("withdraw_client: Could not withdraw. Depositing instead.");
            c.zero().sel1().send(50).recv().0.zero().skip3().close();
        }
    }
}

fn main() {
    let (atm_chan, client_chan) = session_channel();
    spawn(|| atm(atm_chan));
    deposit_client(client_chan);

    let (atm_chan, client_chan) = session_channel();
    spawn(|| atm(atm_chan));
    withdraw_client(client_chan);
}
