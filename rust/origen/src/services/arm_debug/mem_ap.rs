use crate::{Result, Error, TEST, Dut, add_reg_32bit, some_hard_reset_val, field, get_reg};
use num_bigint::BigUint;
use std::sync::MutexGuard;
use crate::core::model::registers::BitCollection;
use super::super::super::services::Service;
use crate::Transaction;

use crate::generator::ast::*;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemAP {
    id: usize,
    arm_debug_id: usize,
    dp_id: usize,
    address_block_id: usize,
    model_id: usize,
    addr: usize,
}

impl MemAP {
    pub fn model_init(dut: &mut crate::Dut, services: &mut crate::Services, model_id: usize, arm_debug_id: usize, addr: usize) -> Result<usize> {
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
                field!("Undefined2", 16, 7, "RW", vec!(), None, ""),
                field!("Type,", 12, 4, "RW", vec!(), None, ""),
                field!("Mode", 8, 4, "RW", vec!(), None, ""),
                field!("TrInProg", 7, 1, "RO", vec!(), None, ""),
                field!("DeviceEn", 6, 1, "RO", vec!(), None, ""),
                field!("AddrInc", 4, 2, "RW", vec!(), None, ""),
                field!("Undefined1", 3, 1, "RW", vec!(), None, ""),
                field!("Size", 0, 3, "RW", vec!(), None, "")
            ),
            "Configures and controls accesses through the MEM-AP to or from a connected memory system."
        );
        add_reg_32bit!(dut, ab_id, "tar", 0x4, Some("RW"), None,
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
                field!("AP_Identification_Variant", 4, 4, "RO", vec!(), None, ""),
                field!("AP_Identification_Type", 0, 3, "RO", vec!(), None, "")
            ),
            "Identifies the Access Port. An IDR value of zero indicates that there is no AP present."
        );

        let id = services.next_id();
        let dp_id = services.get_as_arm_debug(arm_debug_id)?.dp_id()?;
        services.push_service(Service::ArmDebugMemAP(
            Self {
                id: id,
                arm_debug_id: arm_debug_id,
                address_block_id: ab_id,
                dp_id: dp_id,
                model_id: model_id,
                addr: addr,
            }
        ));
        Ok(id)
    }

    pub fn prep_for_transfer<'a, 'b>(&'a self, transaction: &Transaction, dut: &'b MutexGuard<'b, Dut>, services: &crate::Services) -> Result<BitCollection<'b>> {
        let reg = get_reg!(dut, self.address_block_id, "csw");
        let bc = reg.bits(dut);
        bc.set_data(BigUint::from(0x2300_0012 as u32));
        self.write_register(dut, services, &bc)?;

        let reg = get_reg!(dut, self.address_block_id, "tar");
        let bc = reg.bits(dut);
        bc.set_data(BigUint::from(transaction.addr()?));
        self.write_register(dut, services, &bc)?;

        let drw = get_reg!(dut, self.address_block_id, "drw");
        let bc = drw.bits(dut);
        bc.set_data(transaction.data.clone());
        Ok(bc)
    }

    pub fn write_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, bc: &BitCollection) -> Result<()> {
        let reg_write_node = bc.to_write_node(dut)?.unwrap();
        let n_id = TEST.push_and_open(reg_write_node.clone());
        let arm_debug = services.get_as_arm_debug(self.arm_debug_id)?;
        let swd_id = arm_debug.swd_id.unwrap();
        let swd = services.get_as_swd(swd_id)?;
        match &reg_write_node.attrs {
            Attrs::RegWrite(reg_trans) => {
                let reg_id = reg_trans.reg_id.unwrap();
                let reg = dut.get_register(reg_id)?;
                let trans = bc.to_write_transaction(dut)?;
                let addr = trans.addr()?;

                if reg.address_block_id == self.address_block_id {
                    let trans_node = node!(
                        ArmDebugMemAPWriteInternalReg,
                        self.id,
                        self.addr,
                        trans.clone(),
                        None // metadata
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    {
                        let dp = services.get_as_dp(self.dp_id)?;
                        dp.update_select(dut, services, (addr & 0xFFFF_FFF0) as usize)?;
                    }
                    swd.write_ap(dut, trans, crate::swd_ok!())?;

                    TEST.close(trans_node_id)?;
                } else {
                    let trans_node = node!(
                        ArmDebugMemAPWriteReg,
                        self.id,
                        self.addr,
                        trans.clone(),
                        None // metadata
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    let trans = bc.to_write_transaction(dut)?;
                    let drw_bits = self.prep_for_transfer(&trans, dut, services)?;
                    self.write_register(dut, services, &drw_bits)?;
                    TEST.close(trans_node_id)?;
                }
            },
            _ => return Err(Error::new(&format!("Unexpected node in ArmDebug MemAP driver: {:?}", reg_write_node)))
        }
        TEST.close(n_id)?;
        swd.update_actions(dut)?;
        Ok(())
    }

    pub fn verify_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, bc: &BitCollection) -> Result<()> {
        let trans_node;
        let reg_verify_node = bc.to_verify_node(None, true, dut)?.unwrap();
        let n_id = TEST.push_and_open(reg_verify_node.clone());

        let arm_debug = services.get_as_arm_debug(self.arm_debug_id)?;
        let swd_id = arm_debug.swd_id.unwrap();
        let swd = services.get_as_swd(swd_id)?;
        match &reg_verify_node.attrs {
            Attrs::RegVerify(reg_trans) => {
                let reg_id = reg_trans.reg_id.unwrap();
                let reg = dut.get_register(reg_id)?;
                let mut trans = bc.to_verify_transaction(None, false, dut)?;
                if reg.address_block_id == self.address_block_id {
                    // Internal (to the MemAP) register
                    let addr = trans.addr()?;
                    trans_node = node!(
                        ArmDebugMemAPVerifyInternalReg,
                        self.id,
                        self.addr,
                        trans.clone(),
                        None // metadata
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    {
                        let dp = services.get_as_dp(self.dp_id)?;
                        dp.update_select(dut, services, (addr & 0xFFFF_FFF0) as usize)?;
                    }
                    swd.verify_ap(
                        dut,
                        trans.to_dummy()?,
                        crate::swd_ok!(),
                        None
                    )?;

                    swd.verify_ap(
                        dut,
                        trans,
                        crate::swd_ok!(),
                        None
                    )?;
                    TEST.close(trans_node_id)?;
                } else {
                    // External (to the MemAP) register - that is, part of the register map
                    trans_node = node!(
                        ArmDebugMemAPVerifyReg,
                        self.id,
                        self.addr,
                        bc.to_verify_transaction(None, false, dut)?,
                        None
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    self.prep_for_transfer(&trans, dut, services)?;

                    let arm_debug = services.get_as_arm_debug(self.arm_debug_id)?;
                    let swd_id = arm_debug.swd_id.unwrap();
                    let swd = services.get_as_swd(swd_id)?;
                    trans.address = Some(0xC); // DRW
                    swd.verify_ap(
                        dut,
                        trans.to_dummy()?,
                        crate::swd_ok!(),
                        None
                    )?;
                    swd.swdio.drive_low().cycle();
                    trans.address = Some(0xC); // RDBUFF
                    swd.verify_dp(
                        dut,
                        trans,
                        crate::swd_ok!(),
                        None
                    )?;
                    swd.swdio.drive_low().cycle();
                    TEST.close(trans_node_id)?;
                }
            },
            _ => return Err(Error::new(&format!("Unexpected node in ArmDebug MemAP driver: {:?}", reg_verify_node)))
        }
        TEST.close(n_id)?;
        swd.update_actions(dut)?;
        Ok(())
    }
}