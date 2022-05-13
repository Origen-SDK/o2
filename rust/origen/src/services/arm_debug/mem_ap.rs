use super::super::super::services::Service;
use crate::core::model::pins::PinCollection;
use crate::generator::PAT;
use crate::Transaction;
use crate::{add_reg_32bit, field, get_reg, some_hard_reset_val, Dut, Result, TEST};
use num_bigint::BigUint;
use std::sync::MutexGuard;

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
    pub fn model_init(
        dut: &mut crate::Dut,
        services: &mut crate::Services,
        model_id: usize,
        arm_debug_id: usize,
        addr: usize,
    ) -> Result<usize> {
        // Create the model
        let memory_map_id = dut.create_memory_map(model_id, "default", None)?;
        let ab_id =
            dut.create_address_block(memory_map_id, "default", None, None, Some(32), None)?;

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
        add_reg_32bit!(
            dut,
            ab_id,
            "tar",
            0x4,
            Some("RW"),
            None,
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
        services.push_service(Service::ArmDebugMemAP(Self {
            id: id,
            arm_debug_id: arm_debug_id,
            address_block_id: ab_id,
            dp_id: dp_id,
            model_id: model_id,
            addr: addr,
        }));
        Ok(id)
    }

    pub fn prep_for_transfer<'a, 'b>(
        &'a self,
        transaction: &Transaction,
        dut: &'b MutexGuard<'b, Dut>,
        services: &crate::Services,
    ) -> Result<Transaction> {
        let reg = get_reg!(dut, self.address_block_id, "csw");
        let bc = reg.bits(dut);
        bc.set_data(BigUint::from(0x2300_0012 as u32));
        self.write_register(dut, services, &bc.to_write_transaction(dut)?)?;

        let reg = get_reg!(dut, self.address_block_id, "tar");
        let bc = reg.bits(dut);
        bc.set_data(BigUint::from(transaction.addr()?));
        self.write_register(dut, services, &bc.to_write_transaction(dut)?)?;

        let drw = get_reg!(dut, self.address_block_id, "drw");
        let bc = drw.bits(dut);
        bc.set_data(transaction.data.clone());
        Ok(bc.to_write_transaction(dut)?)
    }

    pub fn write_register(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        t: &Transaction,
    ) -> Result<()> {
        let reg_write_node = t.as_write_node()?;
        let n_id = TEST.push_and_open(reg_write_node.clone());
        let arm_debug = services.get_as_arm_debug(self.arm_debug_id)?;

        let jtag_dp;
        let swd_service;
        let jtag_dp_service;
        let swd;
        if *arm_debug.jtagnswd.read().unwrap() {
            jtag_dp_service = services.get_service(arm_debug.jtag_dp_id.expect(
                "Backend Error - JTAG DP requested but a JTAG DP instance is not available (this should have been caught upstream, please review)"
            ))?;
            jtag_dp = Some(jtag_dp_service.as_jtag_dp()?);
            swd = None;
        } else {
            swd_service = services.get_service(arm_debug.swd_id.expect(
                "Backend Error - SWD requested but a SWD instance is not available (this should have been caught upstream, please review)"
            ))?;
            swd = Some(swd_service.as_swd()?);
            jtag_dp = None;
        }

        match &reg_write_node.attrs {
            PAT::RegWrite(reg_trans) => {
                let reg_id = reg_trans.reg_id.unwrap();
                let reg = dut.get_register(reg_id)?;
                let trans = t.clone();
                let addr = trans.addr()?;

                if reg.address_block_id == self.address_block_id {
                    let trans_node = node!(
                        PAT::ArmDebugMemAPWriteInternalReg,
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

                    if jtag_dp.is_some() {
                        jtag_dp.unwrap().write_ap(dut, services, trans)?;
                    } else {
                        swd.unwrap().write_ap(dut, trans, crate::swd_ok!())?;
                    }

                    TEST.close(trans_node_id)?;
                } else {
                    let trans_node = node!(
                        PAT::ArmDebugMemAPWriteReg,
                        self.id,
                        self.addr,
                        trans.clone(),
                        None // metadata
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    let trans = t.clone();
                    let drw_bits = self.prep_for_transfer(&trans, dut, services)?;
                    self.write_register(dut, services, &drw_bits)?;
                    TEST.close(trans_node_id)?;
                }
            }
            _ => {
                bail!(
                    "Unexpected node in ArmDebug MemAP driver: {:?}",
                    reg_write_node
                )
            }
        }
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_register(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        t: &Transaction,
    ) -> Result<()> {
        let trans_node;
        let reg_node;
        if t.is_capture() {
            reg_node = t.as_capture_node()?;
        } else {
            reg_node = t.as_verify_node()?;
        }
        let n_id = TEST.push_and_open(reg_node.clone());

        let arm_debug = services.get_as_arm_debug(self.arm_debug_id)?;
        let jtag_dp;
        let swd_service;
        let jtag_dp_service;
        let swd;
        if *arm_debug.jtagnswd.read().unwrap() {
            jtag_dp_service = services.get_service(arm_debug.jtag_dp_id.expect(
                "Backend Error - JTAG DP requested but a JTAG DP instance is not available (this should have been caught upstream, please review)"
            ))?;
            jtag_dp = Some(jtag_dp_service.as_jtag_dp()?);
            swd = None;
        } else {
            swd_service = services.get_service(arm_debug.swd_id.expect(
                "Backend Error - SWD requested but a SWD instance is not available (this should have been caught upstream, please review)"
            ))?;
            swd = Some(swd_service.as_swd()?);
            jtag_dp = None;
        }

        match &reg_node.attrs {
            PAT::RegVerify(reg_trans) | PAT::RegCapture(reg_trans) => {
                let reg_id = reg_trans.reg_id.unwrap();
                let reg = dut.get_register(reg_id)?;
                let mut trans = t.clone();
                if reg.address_block_id == self.address_block_id {
                    // Internal (to the MemAP) register
                    let addr = trans.addr()?;
                    trans_node = node!(
                        PAT::ArmDebugMemAPVerifyInternalReg,
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
                    if jtag_dp.is_some() {
                        jtag_dp
                            .unwrap()
                            .verify_ap(dut, services, trans.to_dummy()?, true)?;
                        crate::testers::vector_based::api::repeat(100);

                        jtag_dp.unwrap().verify_ap(dut, services, trans, false)?;
                    } else {
                        swd.unwrap()
                            .verify_ap(dut, trans.to_dummy()?, crate::swd_ok!(), None)?;

                        swd.unwrap().verify_ap(dut, trans, crate::swd_ok!(), None)?;
                    }
                    TEST.close(trans_node_id)?;
                } else {
                    // External (to the MemAP) register - that is, part of the register map
                    trans_node = node!(
                        PAT::ArmDebugMemAPVerifyReg,
                        self.id,
                        self.addr,
                        t.clone(),
                        None
                    );
                    let trans_node_id = TEST.push_and_open(trans_node);
                    self.prep_for_transfer(&trans, dut, services)?;

                    if jtag_dp.is_some() {
                        trans.address = Some(BigUint::from(0xC as u32));
                        jtag_dp
                            .unwrap()
                            .verify_ap(dut, services, trans.to_dummy()?, true)?;
                        crate::testers::vector_based::api::repeat(100);

                        trans.address = Some(BigUint::from(0xC as u32));
                        jtag_dp.unwrap().verify_dp(dut, services, trans, false)?;
                    } else {
                        let swdio = PinCollection::from_group(
                            dut,
                            &swd.unwrap().swdio.0,
                            swd.unwrap().swdio.1,
                        )?;
                        trans.address = Some(BigUint::from(0xC as u32)); // DRW
                        swd.unwrap()
                            .verify_ap(dut, trans.to_dummy()?, crate::swd_ok!(), None)?;
                        swdio.drive_low().cycle();
                        trans.address = Some(BigUint::from(0xC as u32)); // RDBUFF
                        swd.unwrap().verify_dp(dut, trans, crate::swd_ok!(), None)?;
                        swdio.drive_low().cycle();
                    }
                    TEST.close(trans_node_id)?;
                }
            }
            _ => {
                bail!("Unexpected node in ArmDebug MemAP driver: {:?}", reg_node)
            }
        }
        TEST.close(n_id)?;
        Ok(())
    }
}
