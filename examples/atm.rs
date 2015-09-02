#[macro_use] extern crate session_types;
use session_types::*;
use std::thread::spawn;

type Id = String;
type Atm = Recv<Id, Choose<(Rec<AtmInner>, Eps)>>;

type AtmInner = Offer<(AtmDeposit,
                       AtmWithdraw,
                       Eps)>;

type AtmDeposit = Recv<u64, Send<u64, Var<Z>>>;
type AtmWithdraw = Recv<u64, Choose<(Var<Z>, Var<Z>)>>;

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
        c = match c.offer() {
            Branch3::B1(c) => {
                let (c, amt) = c.recv();
                balance = amt;
                c.send(balance).zero()   // c.send(new_bal): Chan<(AtmInner, ()) Var<Z>>
            },
            Branch3::B2(c) => {
                let (c, amt) = c.recv();
                if amt <= balance {
                    balance = balance - amt;
                    c.sel1().zero()
                } else {
                    c.sel2().zero()
                }
            },
            Branch3::B3(c) => { c.close(); break }
        };
    }
}

fn deposit_client(c: Chan<(), Client>) {
    let c = match c.send("Deposit Client".to_string()).offer() {
        B1(c) => c.enter(),
        B2(_) => panic!("deposit_client: expected to be approved")
    };

    let (c, new_balance) = c.sel1().send(200).recv();
    println!("deposit_client: new balance: {}", new_balance);
    c.zero().sel3().close();
}

fn withdraw_client(c: Chan<(), Client>) {
    let c = match c.send("Withdraw Client".to_string()).offer() {
        B1(c) => c.enter(),
        B2(_) => panic!("withdraw_client: expected to be approved")
    };

    match c.sel2().send(100).offer() {
        B1(c) => {
            println!("withdraw_client: Successfully withdrew 100");
            c.zero().sel3().close();
        }
        B2(c) => {
            println!("withdraw_client: Could not withdraw. Depositing instead.");
            c.zero().sel1().send(50).recv().0.zero().sel3().close();
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
