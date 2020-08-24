use crate::{Result, TEST, Dut, add_reg_32bit, some_hard_reset_val, field};
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
        let memory_map_id = dut.create_memory_map(model_id, "default", None)?;
        let ab_id = dut.create_address_block(
            memory_map_id,
            "default",
            None,
            None,
            Some(32),
            None
        )?;

        add_reg_32bit!(dut, ab_id, "csw", 0, Some("RW"), some_hard_reset_val!(0),
            vec!(
                field!("DbgSwEnable,", 31, 1, "RW", vec!(), None, ""),
                field!("Prot,", 24, 7, "RW", vec!(), None, ""),
                field!("SPIDEN", 23, 1, "RO", vec!(), None, ""),
                field!("Type,", 12, 4, "RW", vec!(), None, ""),
                field!("Mode", 8, 4, "RW", vec!(), None, ""),
                field!("TrInProg", 7, 1, "RO", vec!(), None, ""),
                field!("DeviceEn", 6, 1, "RO", vec!(), None, ""),
                field!("AddrInc", 4, 2, "RW", vec!(), None, ""),
                field!("Size", 0, 3, "RW", vec!(), None, "")
            ),
            "Configures and controls accesses through the MEM-AP to or from a connected memory system."
        );
        add_reg_32bit!(dut, ab_id, "tar", 0x8, Some("RW"), None,
            vec!(field!("Address", 0, 32, "RW", vec!(), None, "")),
            "Holds the memory address to be accessed."
        );
        add_reg_32bit!(dut, ab_id, "drw", 0xC, Some("RW"), some_hard_reset_val!(0),
            vec!(field!("Data", 0, 32, "RW", vec!(), None, "")),
            "Maps an AP access directly to one or more memory accesses. The AP access does not complete until the memory access, or accesses, complete."
        );
        add_reg_32bit!(dut, ab_id, "idr", 0xFC, Some("RO"), some_hard_reset_val!(0),
            vec!(
                field!("Revision", 28, 4, "RO", vec!(), None, ""),
                field!("Continuation_code", 24, 4, "RO", vec!(), None, ""),
                field!("Identity_code", 17, 7, "RO", vec!(), None, ""),
                field!("Class", 13, 4, "RO", vec!(), None, ""),
                field!("AP_Identificaton_Variant", 4, 4, "RO", vec!(), None, ""),
                field!("AP_Identificaton_Type", 0, 3, "RO", vec!(), None, "")
            ),
            "Identifies the Access Port. An IDR value of zero indicates that there is no AP present."
        );

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

    pub fn verify_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
        let mem_ap_node = TEST.push_and_open(node!(ArmDebugMemAPWriteReg, self.clone()));
        let reg_verify_node = bc.verify(None, true, dut).unwrap().unwrap();
        TEST.close(reg_verify_node)?;
        TEST.close(mem_ap_node)?;
        Ok(())
    }
}