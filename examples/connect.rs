extern crate "rust-sessions" as sessions;
use sessions::*;

fn server(c: Chan<(), Eps>) {
    c.close()
}

fn client(c: Chan<(), Eps>) {
    c.close()
}

fn main() {
    connect(server, client);
}
