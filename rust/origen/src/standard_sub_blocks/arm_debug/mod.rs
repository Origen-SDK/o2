pub mod mem_ap;
pub mod process_transactions;

// Register descriptions taken from the Arm Debug RM
use crate::Result;

#[derive(Clone, Debug)]
pub struct ArmDebug {
    // This SWD instance's model ID
    pub model_id: usize,
    parent_id: usize,
    memory_map_id: usize,
    address_block_id: usize,

    // Arm debug only support JTAG or SWD.
    // Store this just as a bool:
    //  true -> JTAG
    //  false -> SWD
    pub protocol: bool,
}

impl ArmDebug {

    pub fn model_init(dut: &mut crate::Dut, model_id: usize) -> Result<()> {
        Ok(())
    }

    // pub fn model(&self) -> &crate::core::model::Model {
    //     // ...
    // }

    // pub fn protocol(&self) -> Service {
    //     // ...
    // }
}