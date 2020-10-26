use super::super::super::services::Service;
use crate::core::model::registers::BitCollection;
use crate::Transaction;
use num_bigint::BigUint;
use std::sync::MutexGuard;

use crate::generator::ast::Node;

// Register descriptions taken from the Arm Debug RM
use crate::{
    add_reg_32bit, field, get_bc_for, get_reg, get_reg_as_bc, some_hard_reset_val, Dut, Result,
    TEST,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DP {
    model_id: usize,
    memory_map_id: usize,
    address_block_id: usize,
    arm_debug_id: usize,
    id: usize,
}

impl DP {
    pub fn model_init(
        dut: &mut crate::Dut,
        services: &mut crate::Services,
        model_id: usize,
        arm_debug_id: usize,
    ) -> Result<usize> {
        let memory_map_id = dut.create_memory_map(model_id, "default", None)?;
        let ab_id =
            dut.create_address_block(memory_map_id, "default", None, None, Some(32), None)?;

        // Read request -> IDCODE
        // Write request -> ABORT
        add_reg_32bit!(
            dut,
            ab_id,
            "idcode",
            0,
            Some("RW"),
            some_hard_reset_val!(0),
            vec!(
                field!("REVISION", 28, 4, "RO", vec!(), None, ""),
                field!("PARTNO", 20, 8, "RO", vec!(), None, ""),
                field!("MIN,", 16, 1, "RO", vec!(), None, ""),
                field!("VERSION", 12, 4, "RO", vec!(), None, ""),
                field!("DESIGNER", 1, 11, "RO", vec!(), None, ""),
                field!("RES", 0, 1, "RO", vec!(), some_hard_reset_val!(0), "")
            ),
            "Provides information about the Debug Port."
        );
        add_reg_32bit!(dut, ab_id, "abort", 0, Some("RO"), None, vec!(), "");

        add_reg_32bit!(
            dut,
            ab_id,
            "ctrlstat",
            0x4,
            None,
            some_hard_reset_val!(0),
            vec!(
                field!(
                    "CSYSPWRUPACK",
                    31,
                    1,
                    "RO",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
                field!(
                    "CSYSPWRUPREQ",
                    30,
                    1,
                    "RW",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
                field!(
                    "CDBGPWRUPACK",
                    29,
                    1,
                    "RO",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
                field!(
                    "CDBGPWRUPREQ",
                    28,
                    1,
                    "RW",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
                field!(
                    "CDBGRSTACK",
                    27,
                    1,
                    "RO",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
                field!(
                    "CDBGRSTREQ",
                    26,
                    1,
                    "RW",
                    vec!(),
                    some_hard_reset_val!(0),
                    ""
                ),
            ),
            ""
        );
        add_reg_32bit!(
            dut,
            ab_id,
            "dlcr",
            0x4,
            None,
            some_hard_reset_val!(0),
            vec!(),
            ""
        );
        add_reg_32bit!(
            dut,
            ab_id,
            "targetid",
            0x4,
            None,
            some_hard_reset_val!(0),
            vec!(),
            ""
        );
        add_reg_32bit!(
            dut,
            ab_id,
            "dlpidr",
            0x4,
            None,
            some_hard_reset_val!(0),
            vec!(),
            ""
        );
        add_reg_32bit!(
            dut,
            ab_id,
            "eventstat",
            0x4,
            None,
            some_hard_reset_val!(0),
            vec!(),
            ""
        );

        add_reg_32bit!(
            dut,
            ab_id,
            "select",
            0x8,
            Some("RW"),
            some_hard_reset_val!(0),
            vec!(field!(
                "SELECT",
                0,
                32,
                "RW",
                vec!(),
                some_hard_reset_val!(0),
                ""
            )),
            ""
        );

        add_reg_32bit!(
            dut,
            ab_id,
            "rdbuff",
            0xC,
            Some("RO"),
            some_hard_reset_val!(0),
            vec!(field!(
                "DATA",
                0,
                32,
                "RO",
                vec!(),
                some_hard_reset_val!(0),
                ""
            )),
            ""
        );

        let id = services.next_id();
        let arm_debug_id = services.get_as_arm_debug(arm_debug_id)?.id;
        services.push_service(Service::ArmDebugDP(Self {
            model_id: model_id,
            memory_map_id: memory_map_id,
            address_block_id: ab_id,
            arm_debug_id: arm_debug_id,
            id: id,
        }));
        Ok(id)
    }

    pub fn power_up(&self, dut: &mut MutexGuard<Dut>, services: &crate::Services) -> Result<()> {
        // Set the ctrl stat bits
        let reg = get_reg!(dut, self.address_block_id, "ctrlstat");
        let bc = get_bc_for!(dut, reg, "CSYSPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        let bc = get_bc_for!(dut, reg, "CDBGPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        let bc = reg.bits(dut);
        self.write_register(dut, services, &bc)?;
        Ok(())
    }

    pub fn verify_powered_up(
        &self,
        dut: &mut MutexGuard<Dut>,
        services: &crate::Services,
    ) -> Result<()> {
        let reg = get_reg!(dut, self.address_block_id, "ctrlstat");

        let bc = get_bc_for!(dut, reg, "CSYSPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;
        let bc = get_bc_for!(dut, reg, "CDBGPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;

        let bc = get_bc_for!(dut, reg, "CSYSPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;
        let bc = get_bc_for!(dut, reg, "CDBGPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;
        let bc = get_bc_for!(dut, self.address_block_id, "ctrlstat", "CSYSPWRUPACK")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;
        let bc = get_bc_for!(dut, self.address_block_id, "ctrlstat", "CDBGPWRUPACK")?;
        bc.set_data(BigUint::from(1 as u8));
        bc.set_verify_flag(None)?;
        let bc = reg.bits(dut);
        self.verify_register(dut, services, &bc)?;
        bc.clear_flags();
        Ok(())
    }

    pub fn update_select(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        select: usize,
    ) -> Result<Option<Vec<Node>>> {
        let bc = get_bc_for!(dut, self.address_block_id, "select", "SELECT")?;
        let sel = BigUint::from(select);
        if bc.data()? == sel {
            // Select is already at the desired value - can skip this.
            return Ok(None);
        } else {
            bc.set_data(sel);
            self.write_register(dut, services, &bc)?;
            Ok(None)
        }
    }

    pub fn write_register(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        bc: &BitCollection,
    ) -> Result<()> {
        self.reg_trans(dut, services, bc, false)
    }

    pub fn verify_register(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        bc: &BitCollection,
    ) -> Result<()> {
        self.reg_trans(dut, services, bc, true)
    }

    /// Writes a DP register - detailed in chapter 2 of the ARM Debug RM.
    /// For a DP transaction:
    ///     * dpnap must be 1 for all transactions
    ///     * The A field are bits 2 and 3 of the register's address.
    ///     * The data must be 32-bits
    ///     * Some registers, such as IDCODE or ABORT share the same address, but differ whether a read or
    ///       write operation is requested.
    ///     * For the group of five registers which share address 0x4:
    ///         * the index (0, 1, 2, 3, 4) is stored in  SELECT.
    ///         * must write SELECT first, then write the DP register
    pub fn reg_trans(
        &self,
        dut: &MutexGuard<Dut>,
        services: &crate::Services,
        bc: &BitCollection,
        readnwrite: bool,
    ) -> Result<()> {
        let ad_service = services.get_service(self.arm_debug_id)?;
        let arm_debug = ad_service.as_arm_debug()?;
        //let jtag;
        let swd_service;
        // let jtag_service;
        let swd;
        if *arm_debug.jtagnswd.read().unwrap() {
            // Temporary panic just to kill the process - JTAG isn't supported at all rn.
            //  - coreyeng
            panic!("JTAG not supported yet!");
        } else {
            swd_service = services.get_service(arm_debug.swd_id.expect(
                "Backend Error - SWD requested but an SWD instance is not available (this should have been caught upstream, please review)"
            ))?;
            swd = swd_service.as_swd()?;
        }

        let reg_name = bc.reg(dut).unwrap().name.clone();
        let mut select = None;
        match reg_name.as_str() {
            "ctrlstat" => {
                select = Some(0);
            }
            "dlcr" => {
                select = Some(1);
            }
            "targetid" => {
                select = Some(2);
            }
            "dlpidr" => {
                select = Some(3);
            }
            "eventstat" => {
                select = Some(4);
            }
            _ => {}
        }
        if let Some(sel) = select {
            let bc = get_reg_as_bc!(dut, self.address_block_id, "select");
            bc.set_data(BigUint::from(1 as u8));
            TEST.push(node!(
                Comment,
                0,
                format!("ArmDebugDP: select {} (DP Addr: {:X})", reg_name, sel)
            ));
            if *arm_debug.jtagnswd.read().unwrap() {
                // ...
            } else {
                swd.write_dp(
                    dut,
                    Transaction::new_write_with_addr(BigUint::from(sel as u32), 32, 0x8)?,
                    crate::swd_ok!(),
                )?;
            }
        }
        if readnwrite {
            swd.verify_dp(
                dut,
                bc.to_verify_transaction(None, true, dut)?,
                crate::swd_ok!(),
                None,
            )?;
        } else {
            swd.write_dp(dut, bc.to_write_transaction(dut)?, crate::swd_ok!())?;
        }
        Ok(())
    }
}
