extern crate rust_sessions;
use rust_sessions::*;

fn server(c: Chan<(), Eps>) {
    c.close()
}

fn client(c: Chan<(), Eps>) {
    c.close()
}

fn main() {
    connect(server, client);
}
