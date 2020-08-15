use crate::{Result, TEST, Dut};
use num_bigint::BigUint;
use std::sync::MutexGuard;

use crate::core::model::registers::bit::{UNDEFINED, ZERO};
use crate::core::model::registers::{Bit, BitOrder, SummaryField};
use std::sync::RwLock;
use std::collections::HashMap;
use crate::core::model::registers::BitCollection;
// use register::{Field, FieldEnum, ResetVal};

macro_rules! reg_32bit {
    ( $dut:expr, $address_block_id:expr, $name:expr, $offset:expr, $description:expr ) => {{
        let base_bit_id;
        {
            base_bit_id = $dut.bits.len();
        }
        let r_id = $dut.create_reg(
            $address_block_id,
            None,
            $name,
            $offset,
            Some(32),
            "LSB0",
            None,
            None,
            Some($description.to_string())
        )?;
        let r = $dut.get_mut_register(r_id)?;
        for i in 0..r.size as usize {
            r.bit_ids.push((base_bit_id + i) as usize);
        }
        r
    }};
}

macro_rules! add_field {
    ( $reg:expr, $name:expr, $offset:expr, $width:expr, $access:expr, $description:expr ) => {{
        $reg.add_field(
            $name,
            Some(&$description.to_string()),
            $offset,
            $width,
            $access,
            None,
            None
        )
    }};
}

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

        // Add the registers
        // let reg_id = dut.create_reg(
        //     ab_id,
        //     None,
        //     "CSW",
        //     0,
        //     Some(32),
        //     "LSB0",
        //     None,
        //     None,
        //     Some("The CSW register configures and controls accesses through the MEM-AP to or from a connected memory system.".to_string())
        // )?;
        let mut r = reg_32bit!(dut, ab_id, "CSW", 0, "The CSW register configures and controls accesses through the MEM-AP to or from a connected memory system.");
        // let mut r = dut.get_mut_register(reg_id)?;
        // r.add_field(
        //     "DbgSwEnable",
        //     Some(&"".to_string()),
        //     31,
        //     1,
        //     "RW",
        //     None,
        //     None
        // )?;
        add_field!(r, "DbgSwEnable", 31, 1, "RW", "")?;
        add_field!(r, "Prot", 24, 7, "RW", "")?;
        add_field!(r, "SPIDEN", 23, 1, "RO", "")?;
        add_field!(r, "Reserved", 16, 7, "RO", "")?;
        add_field!(r, "Type", 12, 3, "RW", "")?;
        add_field!(r, "Mode", 8, 4, "RW", "")?;
        add_field!(r, "TrinProg", 7, 1, "RW", "")?;
        add_field!(r, "DeviceEn", 6, 1, "RW", "")?;
        add_field!(r, "AddInc", 4, 2, "RW", "")?;
        add_field!(r, "Reserved", 3, 1, "RW", "")?;
        add_field!(r, "Size", 0, 3, "RW", "")?;

        let reg_fields = r.fields(true).collect::<Vec<SummaryField>>();
        let reg_id = r.id;
        for field in reg_fields {
            // Intention here is to skip decomposing the BigUint unless required
            // if !non_zero_reset || field.spacer || reset_vals.last().unwrap().is_none() {
                for i in 0..field.width {
                    let val;
                    if field.spacer {
                        val = ZERO;
                    } else {
                        val = UNDEFINED;
                    }
                    let id;
                    {
                        id = dut.bits.len();
                    }
                    dut.bits.push(Bit {
                        id: id,
                        overlay: RwLock::new(None),
                        overlay_snapshots: RwLock::new(HashMap::new()),
                        register_id: reg_id,
                        state: RwLock::new(val),
                        device_state: RwLock::new(val),
                        state_snapshots: RwLock::new(HashMap::new()),
                        access: field.access,
                        position: field.offset + i,
                    });
                }
            // } else {
            //     let reset_val = reset_vals.last().unwrap().as_ref().unwrap();
    
            //     // If no reset mask to apply. There is a lot of duplication here but ran
            //     // into borrow issues that I couldn't resolve and had to move on.
            //     if reset_val.1.as_ref().is_none() {
            //         let mut bytes = reset_val.0.to_bytes_be();
            //         let mut byte = bytes.pop().unwrap();
            //         for i in 0..field.width {
            //             let state = (byte >> i % 8) & 1;
            //             let id;
            //             {
            //                 id = dut.bits.len();
            //             }
            //             dut.bits.push(Bit {
            //                 id: id,
            //                 overlay: RwLock::new(None),
            //                 overlay_snapshots: RwLock::new(HashMap::new()),
            //                 register_id: reg_id,
            //                 state: RwLock::new(state),
            //                 device_state: RwLock::new(state),
            //                 state_snapshots: RwLock::new(HashMap::new()),
            //                 access: field.access,
            //                 position: field.offset + i,
            //             });
            //             if i % 8 == 7 {
            //                 match bytes.pop() {
            //                     Some(x) => byte = x,
            //                     None => byte = 0,
            //                 }
            //             }
            //         }
            //     } else {
            //         let mut bytes = reset_val.0.to_bytes_be();
            //         let mut byte = bytes.pop().unwrap();
            //         let mut mask_bytes = reset_val.1.as_ref().unwrap().to_bytes_be();
            //         let mut mask_byte = mask_bytes.pop().unwrap();
            //         for i in 0..field.width {
            //             let state = (byte >> i % 8) & (mask_byte >> i % 8) & 1;
            //             let id;
            //             {
            //                 id = dut.bits.len();
            //             }
            //             dut.bits.push(Bit {
            //                 id: id,
            //                 overlay: RwLock::new(None),
            //                 overlay_snapshots: RwLock::new(HashMap::new()),
            //                 register_id: reg_id,
            //                 state: RwLock::new(state),
            //                 device_state: RwLock::new(state),
            //                 state_snapshots: RwLock::new(HashMap::new()),
            //                 access: field.access,
            //                 position: field.offset + i,
            //             });
            //             if i % 8 == 7 {
            //                 match bytes.pop() {
            //                     Some(x) => byte = x,
            //                     None => byte = 0,
            //                 }
            //                 match mask_bytes.pop() {
            //                     Some(x) => mask_byte = x,
            //                     None => mask_byte = 0,
            //                 }
            //             }
            //         }
            //     }
            // }
            // if !field.spacer {
            //     reset_vals.pop();
            // }
        }
    

        let mut r = reg_32bit!(dut, ab_id, "TAR", 0x4, "");
        let mut r = reg_32bit!(dut, ab_id, "TAR_LPAE", 0x8, "");
        let mut r = reg_32bit!(dut, ab_id, "DRW", 0xC, "");
        let mut r = reg_32bit!(dut, ab_id, "BD0", 0x10, "");
        let mut r = reg_32bit!(dut, ab_id, "BD1", 0x14, "");
        let mut r = reg_32bit!(dut, ab_id, "BD2", 0x18, "");
        let mut r = reg_32bit!(dut, ab_id, "BD3", 0x1C, "");
        let mut r = reg_32bit!(dut, ab_id, "MBT", 0x20, "");
        let mut r = reg_32bit!(dut, ab_id, "BASE_LPAE", 0xF0, "");
        let mut r = reg_32bit!(dut, ab_id, "CFG", 0xF4, "");
        let mut r = reg_32bit!(dut, ab_id, "BASE", 0xF8, "");
        let mut r = reg_32bit!(dut, ab_id, "IDR", 0xFC, "");
        // Ok(Self {
        //     model_id: model_id,
        //     memory_map_id: memory_map_id,
        //     address_block_id: address_block_id,
        //     parent_id: parent_id,
        //     protocol: true,
        // })
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