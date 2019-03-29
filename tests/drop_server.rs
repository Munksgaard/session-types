extern crate session_types;

use session_types::*;

fn client(c: Chan<(), Send<(), Eps>>) {
    c.send(()).close();
}

fn server(c: Chan<(), Recv<(), Eps>>) {
    let (c, ()) = c.recv();
    c.close();
}

fn drop_client(_c: Chan<(), Send<(), Eps>>) {}

fn drop_server(_c: Chan<(), Recv<(), Eps>>) {}

#[test]
fn server_client_works() {
    connect(server, client);
}

#[test]
#[should_panic]
fn client_incomplete_panics() {
    connect(server, drop_client);
}

#[test]
#[should_panic]
fn server_incomplete_panics() {
    connect(drop_server, client);
}

#[test]
#[should_panic]
fn server_client_incomplete_panics() {
    connect(drop_server, drop_client);
}