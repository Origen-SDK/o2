use crate::{Result, TEST, Dut};
use num_bigint::BigUint;
use std::sync::MutexGuard;
use crate::core::model::registers::BitCollection;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemAP {
    model_id: usize,
    addr: BigUint,
}

impl MemAP {
    pub fn model_init(dut: &mut crate::Dut, model_id: usize, addr: BigUint) -> Result<Self> {
        // Create the model
        let memory_map_id = dut.create_memory_map(model_id, "default", Some(32))?;
        let ab_id = dut.create_address_block(
            memory_map_id,
            "default",
            None,
            None,
            Some(32),
            None
        )?;

        Ok(Self {
            model_id: model_id,
            addr: addr,
        })
    }

    pub fn write_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
        let mem_ap_node = TEST.push_and_open(node!(ArmDebugMemAPWriteReg, self.clone()));
        let reg_write_node = bc.write(dut).unwrap().unwrap();
        TEST.close(reg_write_node)?;
        TEST.close(mem_ap_node)?;
        Ok(())
    }

    // pub fn write(&self, addr: num_bigint::BigUint, data: num_bigint::BigUint) -> Result<()> {
    //     TEST.push(node!(ArmDebugMemAPWrite, self.clone(), addr, data));
    //     Ok(())
    // }

    // pub fn read(&self, addr: num_bigint::BigUint, data: num_bigint::BigUint) -> Result<()> {
    //     TEST.push(node!(ArmDebugMemAPRead, self.clone(), addr, data));
    //     Ok(())
    // }

    // pub fn capture(&self, addr: num_bigint::BigUint) -> Result<()> {
    //     TEST.push(node!(ArmDebugMemAPCapture, self.clone(), addr));
    //     Ok(())
    // }
}