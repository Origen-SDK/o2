pub mod mem_ap;
pub mod process_transactions;
use num_bigint::BigUint;
use std::sync::MutexGuard;
use crate::core::model::registers::BitCollection;

// Register descriptions taken from the Arm Debug RM
use crate::{Result, TEST, Dut, 
    add_reg_32bit, field, some_hard_reset_val, get_bc_for, get_reg_as_bc,
    get_reg
};

macro_rules! extract_reg32_params {
    ( $dut:expr, $bc:expr) => {{
        (
            $bc.reg($dut).unwrap().name.clone(),
            $bc.reg($dut).unwrap().address($dut, Some(32)).unwrap() as u32,
            $bc.data().unwrap(),
        )
    }};
}

#[derive(Clone, Debug)]
pub struct ArmDebug {
    // This SWD instance's model ID
    pub model_id: usize,
    //parent_id: usize,
    // memory_map_id: usize,
    // dp_id: usize,

    // Arm debug only support JTAG or SWD.
    // Store this just as a bool:
    //  true -> JTAG
    //  false -> SWD
    pub jtagdp: bool,
}

impl ArmDebug {

    pub fn model_init(_dut: &mut crate::Dut, model_id: usize) -> Result<Self> {
        // Add the DP as an address block within this instance
        //let mut r = reg_32bit!(dut, dp_id, "IDCODE", 0xC, "");
        Ok(Self {
            model_id: model_id,
            // memory_map_id: memory_map_id,
            // dp_id: dp_id,
            jtagdp: true,
        })
    }

    pub fn switch_to_swd(&mut self, line_reset: bool) -> Result<()> {
        if line_reset {
            crate::TEST.push(crate::node!(SWDLineReset));
        }
        crate::TEST.push(crate::node!(ArmDebugSwjJTAGToSWD, self.model_id));
        self.jtagdp = false;
        Ok(())
    }

    /// Discerns how the register should be written (AP vs. DP) and adds the appropriate nodes
    pub fn write_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
        // ...
        Ok(())
    }

    // pub fn model(&self) -> &crate::core::model::Model {
    //     // ...
    // }

    // pub fn protocol(&self) -> Service {
    //     // ...
    // }
}

#[derive(Clone)]
pub struct DP {
    model_id: usize,
    memory_map_id: usize,
    dp_id: usize,
}

impl DP {
    pub fn model_init(dut: &mut crate::Dut, model_id: usize) -> Result<Self> {
        let memory_map_id = dut.create_memory_map(model_id, "default", Some(32))?;
        let dp_id = dut.create_address_block(
            memory_map_id,
            "default",
            None,
            None,
            Some(32),
            None
        )?;

        // Read request -> IDCODE
        // Write request -> ABORT
        add_reg_32bit!(dut, dp_id, "idcode", 0, Some("RW"), some_hard_reset_val!(0),
            vec!(
                field!("REVISION", 28, 4, "RO", vec!(), None, ""),
                field!("PARTNO", 20, 8, "RO", vec!(), None, ""),
                field!("MIN,", 16, 1, "RO", vec!(), None, ""),
                field!("VERSION", 12, 4, "RO", vec!(), None, ""),
                field!("DESIGNER", 1, 11, "RO", vec!(), None, "")
            ),
            "Provides information about the Debug Port."
        );
        add_reg_32bit!(dut, dp_id, "abort", 0, Some("RO"), None, vec!(), "");

        add_reg_32bit!(dut, dp_id, "ctrlstat", 0x4, None, some_hard_reset_val!(0),
            vec!(
                field!("CSYSPWRUPACK", 31, 1, "RO", vec!(), some_hard_reset_val!(0), ""),
                field!("CSYSPWRUPREQ", 30, 1, "RW", vec!(), some_hard_reset_val!(0), ""),
                field!("CDBGPWRUPACK", 29, 1, "RO", vec!(), some_hard_reset_val!(0), ""),
                field!("CDBGPWRUPREQ", 28, 1, "RW", vec!(), some_hard_reset_val!(0), ""),
                field!("CDBGRSTACK", 27, 1, "RO", vec!(), some_hard_reset_val!(0), ""),
                field!("CDBGRSTREQ", 26, 1, "RW", vec!(), some_hard_reset_val!(0), ""),
            ),
            ""
        );
        add_reg_32bit!(dut, dp_id, "dlcr", 0x4, None, some_hard_reset_val!(0), vec!(), "");
        add_reg_32bit!(dut, dp_id, "targetid", 0x4, None, some_hard_reset_val!(0), vec!(), "");
        add_reg_32bit!(dut, dp_id, "dlpidr", 0x4, None, some_hard_reset_val!(0), vec!(), "");
        add_reg_32bit!(dut, dp_id, "eventstat", 0x4, None, some_hard_reset_val!(0), vec!(), "");

        add_reg_32bit!(dut, dp_id, "select", 0x8, Some("RW"), some_hard_reset_val!(0), vec!(), "");
        Ok(Self {
            model_id: model_id,
            memory_map_id: memory_map_id,
            dp_id: dp_id,
        })
    }

    pub fn power_up(&self, dut: &mut MutexGuard<Dut>) -> Result<()> {
        // Set the ctrl stat bits
        let reg = get_reg!(dut, self.dp_id, "ctrlstat");
        let bc = get_bc_for!(dut, reg, "CSYSPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        let bc = get_bc_for!(dut, reg, "CDBGPWRUPREQ")?;
        bc.set_data(BigUint::from(1 as u8));
        let bc = reg.bits(dut);
        self.write_register(dut, &bc)?;
        Ok(())
    }

    pub fn verify_powered_up(&self, dut: &mut MutexGuard<Dut>) -> Result<()> {
        let bc = get_bc_for!(dut, self.dp_id, "ctrlstat", "CSYSPWRUPACK")?;
        bc.set_data(BigUint::from(1 as u8));
        let bc = get_bc_for!(dut, self.dp_id, "ctrlstat", "CDBGPWRUPACK")?;
        bc.set_data(BigUint::from(1 as u8));
        Ok(())
    }

    pub fn write_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
        self.reg_trans(dut, bc, false)
    }

    pub fn verify_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
        self.reg_trans(dut, bc, true)
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
    pub fn reg_trans(&self, dut: &MutexGuard<Dut>, bc: &BitCollection, readnwrite: bool) -> Result<()> {
        let (reg_name, reg_addr, reg_data) = extract_reg32_params!(dut, bc);
        let dp_addr = (reg_addr & 0xF) >> 2;
        let mut select = None;
        match reg_name.as_str() {
            "ctrlstat" => {
                select = Some(0);
            },
            "dlcr" => {
                select = Some(1);
            },
            "targetid" => {
                select = Some(2);
            },
            "dlpidr" => {
                select = Some(3);
            },
            "eventstat" => {
                select = Some(4);
            }
            _ => {}
        }
        if let Some(sel) = select {
            let bc = get_reg_as_bc!(dut, self.dp_id, "select");
            bc.set_data(BigUint::from(1 as u8));
            TEST.push(node!(SWDWriteDP, BigUint::from(sel as u8), 0x2, crate::swd_ok!()))
        }
        if readnwrite {
            TEST.push(node!(SWDVerifyDP, reg_data, dp_addr, crate::swd_ok!(), None));
        } else {
            TEST.push(node!(SWDWriteDP, reg_data, dp_addr, crate::swd_ok!()));
        }
        Ok(())
    }
}
