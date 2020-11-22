//! The service implements the public API exposed to Python and provides
//! all state storage for a JTAG driver instance

use crate::node;
use crate::precludes::controller::*;
use crate::testers::api::ControllerAPI;
use crate::{Dut, Result, Transaction, TEST};

// pub enum TAPStates {
//     Reset,
//     Idle,

//     // DR States
//     SelectDR,
//     CaptureDR,
//     ShiftDR,
//     Exit1DR,
//     PauseDR,
//     Exit2DR,
//     UpdateDR,

//     // IR States
//     SelectIR,
//     CaptureIR,
//     ShiftIR,
//     Exit1IR,
//     PauseIR,
//     Exit2IR,
//     UpdateIR,
// }

// impl TAPStates{
//     // pub fn cycles_to_idle(&self) -> usize {
//     //     match self {
//     //         Reset => 0,
//     //         Idle => 3,
//     //         _ => 0
//     //     }
//     // }

//     // pub fn next_state(&self, tms_val) -> usize {
//     //     // ...
//     // }

//     // pub fn cycles_to_reset(&self) -> usize {
//     //     self.cycles_to_idle() + 3
//     // }

//     // pub fn to_idle(&self) -> Result<Self> {
//     //     // ...
//     // }

//     // pub fn to_reset(&self) -> Result<Self> {
//     //     // ...
//     // }

//     // pub fn to_shift_ir(&self) -> Result<Self> {
//     //     // ...
//     // }

//     // pub fn to_update_ir(&self) -> Result<Self> {
//     //     // ...
//     // }

//     // pub fn to_shift_dr(&self) -> Result<Self> {
//     //     // ...
//     // }

//     // pub fn to_update_dr(&self) -> Result<Self> {
//     //     // ...
//     // }
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Service {
    // For example, to keep track of the current IR value, would also add fields
    // here to record the pins (defined when the service was instantiated)
    id: usize,
    default_ir_size: Option<usize>,
    tclk: (String, usize),
    tdi: (String, usize),
    tdo: (String, usize),
    tms: (String, usize),
    trstn: (String, usize),
}

impl ControllerAPI for Service {
    fn name(&self) -> String {
        "JTAG".to_string()
    }
}

impl Service {
    pub fn new(
        _dut: &Dut,
        id: usize,
        default_ir_size: Option<usize>,
        tclk: Option<&PinGroup>,
        tdi: Option<&PinGroup>,
        tdo: Option<&PinGroup>,
        tms: Option<&PinGroup>,
        trstn: Option<&PinGroup>,
    ) -> Result<Service> {
        Ok(Service {
            id: id,
            default_ir_size: default_ir_size,
            tclk: {
                if let Some(grp) = tclk {
                    grp.to_identifier()
                } else {
                    ("tclk".to_string(), 0)
                }
            },
            tdi: {
                if let Some(grp) = tdi {
                    grp.to_identifier()
                } else {
                    ("tdi".to_string(), 0)
                }
            },
            tdo: {
                if let Some(grp) = tdo {
                    grp.to_identifier()
                } else {
                    ("tdo".to_string(), 0)
                }
            },
            tms: {
                if let Some(grp) = tms {
                    grp.to_identifier()
                } else {
                    ("tms".to_string(), 0)
                }
            },
            trstn: {
                if let Some(grp) = trstn {
                    grp.to_identifier()
                } else {
                    ("trstm".to_string(), 0)
                }
            },
        })
    }

    pub fn reset(&self, dut: &Dut) -> Result<()> {
        let tms = PinCollection::from_group(&dut, &self.tms.0, self.tms.1)?;
        self._reset(&tms)
    }

    fn _reset(&self, tms: &PinCollection) -> Result<()> {
        let n_id = TEST.push_and_open(node!(JTAGReset, self.id, None));
        self.comment("Resetting JTAG Interface");
        tms.drive_high().repeat(6);
        TEST.close(n_id)?;
        Ok(())
    }

    // pub fn idle(&self) -> Result<()> {
    //     // ...
    // }

    // pub fn move_to_state(&self, state: JTAGState) -> Result<()> {
    //     // ...
    // }

    // fn _move_to_state(&self, state: JTAGState, tms: &PinCollection) -> Result<()> {
    //     // ...
    // }

    pub fn write_ir(&self, dut: &Dut, transaction: &Transaction) -> Result<()> {
        let tms = PinCollection::from_group(&dut, &self.tms.0, self.tms.1)?;
        let tdi = PinCollection::from_group(&dut, &self.tdi.0, self.tdi.1)?;
        let trans = node!(JTAGWriteIR, self.id, transaction.clone(), None);
        let n_id = TEST.push_and_open(trans.clone());

        self.comment("Move to Shift-IR");
        tms.drive_low().cycle();
        tms.drive_high().cycles(2);
        tms.drive_low().cycles(2);
        self.comment(&format!("Write IR {:?}", transaction.data));
        let mut nodes = tdi.push_transaction_nodes(&transaction)?;
        nodes.insert(
            nodes.len() - 2,
            tms.drive_high_nodes().first().unwrap().clone(),
        );
        TEST.append(&mut nodes);
        self.comment("Completed IR Shift");
        tdi.highz();
        self.comment("Move to Update IR");
        tms.drive_high().cycle();
        self.comment("Move to IDLE");
        tms.drive_low().cycle();

        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_ir(&self, dut: &Dut, transaction: &Transaction) -> Result<()> {
        let tms = PinCollection::from_group(&dut, &self.tms.0, self.tms.1)?;
        let tdo = PinCollection::from_group(&dut, &self.tdo.0, self.tdo.1)?;
        let trans = node!(JTAGVerifyIR, self.id, transaction.clone(), None);
        let n_id = TEST.push_and_open(trans.clone());
        self.comment("Move to Shift-IR");
        tms.drive_low().cycle();
        tms.drive_high().cycles(2);
        tms.drive_low().cycles(2);
        self.comment(&format!("Verify IR {:?}", transaction.data));
        let mut nodes = tdo.push_transaction_nodes(&transaction)?;
        nodes.insert(
            nodes.len() - 2,
            tms.drive_high_nodes().first().unwrap().clone(),
        );
        TEST.append(&mut nodes);
        self.comment("Completed IR Shift");
        tdo.highz();
        self.comment("Move to Update IR");
        tms.drive_high().cycle();
        self.comment("Move to IDLE");
        tms.drive_low().cycle();
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write_dr(&self, dut: &Dut, transaction: &Transaction) -> Result<()> {
        let tms = PinCollection::from_group(&dut, &self.tms.0, self.tms.1)?;
        let tdi = PinCollection::from_group(&dut, &self.tdi.0, self.tdi.1)?;
        let trans = node!(JTAGWriteDR, self.id, transaction.clone(), None);
        let n_id = TEST.push_and_open(trans.clone());

        self.comment("Move to Shift-DR");
        tms.drive_low().cycle();
        tms.drive_high().cycle();
        tms.drive_low().cycles(2);
        self.comment(&format!("Write DR {:?}", transaction.data));
        let mut nodes = tdi.push_transaction_nodes(&transaction)?;
        nodes.insert(
            nodes.len() - 2,
            tms.drive_high_nodes().first().unwrap().clone(),
        );
        TEST.append(&mut nodes);
        self.comment("Completed DR Shift");
        tdi.highz();
        self.comment("Move to Update IR");
        tms.drive_high().cycle();
        self.comment("Move to IDLE");
        tms.drive_low().cycle();

        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_dr(&self, dut: &Dut, transaction: &Transaction) -> Result<()> {
        let tms = PinCollection::from_group(&dut, &self.tms.0, self.tms.1)?;
        let tdo = PinCollection::from_group(&dut, &self.tdo.0, self.tdo.1)?;
        let trans = node!(JTAGVerifyDR, self.id, transaction.clone(), None);
        let n_id = TEST.push_and_open(trans.clone());

        self.comment("Move to Shift-DR");
        tms.drive_low().cycle();
        tms.drive_high().cycle();
        tms.drive_low().cycles(2);
        self.comment(&format!("Verify DR {:?}", transaction.data));
        let mut nodes = tdo.push_transaction_nodes(&transaction)?;
        nodes.insert(
            nodes.len() - 2,
            tms.drive_high_nodes().first().unwrap().clone(),
        );
        TEST.append(&mut nodes);
        self.comment("Completed DR Shift");
        tdo.highz();
        self.comment("Move to Update IR");
        tms.drive_high().cycle();
        self.comment("Move to IDLE");
        tms.drive_low().cycle();

        TEST.close(n_id)?;
        Ok(())
    }
}
