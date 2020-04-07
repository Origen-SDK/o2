//! The service implements the public API exposed to Python and provides
//! all state storage for a JTAG driver instance

use crate::node;
use crate::{Result, Value, TEST};

#[derive(Clone, Debug)]
pub struct Service {
    // For example, to keep track of the current IR value, would also add fields
    // here to record the pins (defined when the service was instantiated)
    pub ir_val: u64,
}

fn cycles(num: u32) {
    // This is temporary to create some AST content, the only direct Node creation
    // done by this driver should be JTAG nodes.
    // Cycles will be induced by API method calls in future, e.g. tester.cycle()
    let cyc = node!(Cycle, 1, true);
    for _i in 0..num {
        TEST.push(cyc.clone());
    }
}

impl Service {
    pub fn new() -> Service {
        Service { ir_val: 0 }
    }

    pub fn write_ir(&mut self, value: Value) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, size) => node!(
                JTAGWriteIR,
                match size {
                    None => bits.len() as u32,
                    Some(x) => x,
                },
                bits.data()?,
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, size) => node!(JTAGWriteIR, size, value, None, None),
        };
        let tid = TEST.push_and_open(trans);

        // This is temporary to create some AST content
        cycles(5);

        TEST.close(tid)
            .expect("Closed JTAG write IR trans properly");

        Ok(())
    }

    pub fn verify_ir(&mut self, value: Value) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, size) => node!(
                JTAGVerifyIR,
                match size {
                    None => bits.len() as u32,
                    Some(x) => x,
                },
                bits.data()?,
                Some(bits.verify_enables()),
                Some(bits.capture_enables()),
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, size) => node!(JTAGVerifyIR, size, value, None, None, None, None),
        };
        let tid = TEST.push_and_open(trans);

        // This is temporary to create some AST content
        cycles(15);

        TEST.close(tid)
            .expect("Closed JTAG write IR trans properly");

        Ok(())
    }

    pub fn write_dr(&mut self, value: Value) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, size) => node!(
                JTAGWriteDR,
                match size {
                    None => bits.len() as u32,
                    Some(x) => x,
                },
                bits.data()?,
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, size) => node!(JTAGWriteIR, size, value, None, None),
        };
        let tid = TEST.push_and_open(trans);

        // This is temporary to create some AST content
        cycles(5);

        TEST.close(tid)
            .expect("Closed JTAG write IR trans properly");

        Ok(())
    }

    pub fn verify_dr(&mut self, value: Value) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, size) => node!(
                JTAGVerifyDR,
                match size {
                    None => bits.len() as u32,
                    Some(x) => x,
                },
                bits.data()?,
                Some(bits.verify_enables()),
                Some(bits.capture_enables()),
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, size) => node!(JTAGVerifyIR, size, value, None, None, None, None),
        };
        let tid = TEST.push_and_open(trans);

        // This is temporary to create some AST content
        cycles(15);

        TEST.close(tid)
            .expect("Closed JTAG write IR trans properly");

        Ok(())
    }
}
