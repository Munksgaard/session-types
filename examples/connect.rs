extern crate session_types;
use session_types::*;

fn server(c: Chan<(), Eps>) {
    c.close()
}

fn client(c: Chan<(), Eps>) {
    c.close()
}

fn main() {
    connect(server, client);
}
