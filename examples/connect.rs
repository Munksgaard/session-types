extern crate session_types;
use session_types::*;

fn server<'run>(c: Chan2<'run, (), Eps>) -> Complete<'run, Eps> {
    return c.close()
}

fn client<'run>(c: Chan2<'run, (), Eps>) -> Complete<'run, Eps> {
    return c.close()
}

fn main() {
    connect(server, client);
}
