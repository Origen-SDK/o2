//! The service implements the public API exposed to Python and provides
//! all state storage for a JTAG driver instance

use crate::generator::ast::*;
use crate::TEST;

#[derive(Clone, Debug)]
pub struct Service {}

impl Service {
    pub fn new() -> Service {
        Service {}
    }

    pub fn write_ir(&mut self) {
        println!("Write IR");
        let trans = Node::new(Attrs::JTAGWriteIR);
        let tid = TEST.push_and_open(trans);

        let cyc = Node::new(Attrs::Cycle(1, true));
        for _i in 0..5 {
            TEST.push(cyc.clone());
        }
        TEST.close(tid).expect("Closed reg trans properly");
    }
}
