extern crate session_types;

use session_types::HasDual;

struct CustomProto;

impl HasDual for CustomProto { //~ ERROR the trait bound `CustomProto: session_types::private::Sealed` is not satisfied
    type Dual = CustomProto;
}

fn main() {}
