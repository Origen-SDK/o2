use crate::{Result, Error, TEST, Dut, Services,
    add_reg_32bit, add_reg, field, some_hard_reset_val, get_bc_for, get_reg_as_bc,
    get_reg
};
use num_bigint::BigUint;
use std::sync::MutexGuard;
use crate::core::model::registers::BitCollection;
use super::super::super::services::Service;
//use crate::core::dut::Dut;
use crate::precludes::controller::*;

#[derive(Clone, Debug)]
pub struct JtagDP {
    pub id: usize,
    model_id: usize,
    memory_map_id: usize,
    address_block_id: usize,
    arm_debug_id: usize,
    // dp_id: usize,
    pub jtag_id: usize,
    pub default_ir_size: usize,
    pub dpacc_select: u32,
    pub apacc_select: u32,
}

impl JtagDP {
    pub fn model_init(
        dut: &mut Dut,
        services: &mut Services,
        model_id: usize,
        arm_debug_id: usize,
        default_ir_size: Option<usize>,
        default_idcode: Option<u32>,
        dpacc_select: Option<u32>,
        apacc_select: Option<u32>
    ) -> Result<usize> {
        // Add the JTAG DP registers
        let memory_map_id = dut.create_memory_map(model_id, "default", None)?;
        let ab_id = dut.create_address_block(
            memory_map_id,
            "default",
            None,
            None,
            Some(32),
            None
        )?;
        add_reg_32bit!(dut, ab_id, "idcode", 0xE, Some("RO"),
            some_hard_reset_val!(default_idcode.unwrap_or(0x477) as u128),
            vec!(
                field!("VERSION", 28, 4, "RO", vec!(), None, ""),
                field!("PARTNO", 12, 16, "RO", vec!(), None, ""),
                field!("CONTCODE", 8, 4, "RO", vec!(), None, "Continuation Code"),
                field!("IDCODE", 1, 7, "RO", vec!(), None, "Identity Code"),
                field!("RES", 0, 1, "RO", vec!(), None, "")
            ),
            ""
        );
        add_reg!(dut, ab_id, "dpacc", 0xA, 35, Some("RW"),
            some_hard_reset_val!(0),
            vec!(
                field!("data", 3, 32, "RW", vec!(), None, ""),
                field!("a", 1, 2, "RW", vec!(), None, ""),
                field!("rnw", 0, 1, "RW", vec!(), None, ""),
            ),
            ""
        );
        add_reg!(dut, ab_id, "apacc", 0xB, 35, Some("RW"),
            some_hard_reset_val!(0),
            vec!(
                field!("data", 3, 32, "RW", vec!(), None, ""),
                field!("a", 1, 2, "RW", vec!(), None, ""),
                field!("rnw", 0, 1, "RW", vec!(), None, ""),
            ),
            ""
        );

        let id = services.next_id();
        let arm_debug = services.get_as_arm_debug(arm_debug_id)?;
        services.push_service(Service::ArmDebugJtagDP(
            Self {
                id: id,
                model_id: model_id,
                memory_map_id: memory_map_id,
                address_block_id: ab_id,
                arm_debug_id: arm_debug.id,
                // dp_id: arm_debug.dp_id,
                jtag_id: {
                    if let Some(j_id) = arm_debug.jtag_id {
                        j_id
                    } else {
                        return Err(Error::new("Arm Debug's JTAG DP requires a JTAG service. No JTAG service was given"));
                    }
                },
                default_ir_size: default_ir_size.unwrap_or(4),
                dpacc_select: dpacc_select.unwrap_or(0xA),
                apacc_select: apacc_select.unwrap_or(0xB)
            }
        ));
        Ok(id)
    }

    pub fn prep_transaction(&self, trans: &mut Transaction, rnw: bool) -> Result<()> {
        let d = BigUint::from((trans.addr()? & 0xC) << 1);
        if rnw {
            trans.prepend_data(d + (1 as u8), 3)?;
        } else {
            trans.prepend_data(d, 3)?;
        }
        Ok(())
    }

    pub fn update_ir(&self, dut: &Dut, services: &Services, ir_value: u32) -> Result<()> {
        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        let ir_trans = Transaction::new_write(BigUint::from(ir_value), self.default_ir_size)?;
        jtag.write_ir(&dut, &ir_trans)?;
        Ok(())
    }

    pub fn write_dp(&self, dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction) -> Result<()> {
        // let jtag_service = services.get_service(self.jtag_id)?;
        // let jtag = jtag_service.as_jtag()?;
        // let mut trans = node!(
        //     ArmDebugJTAGWriteDP,
        //     self.id,
        //     transaction.clone(),
        //     None
        // );
        // let n_id = TEST.push_and_open(trans.clone());

        let reg = get_reg!(dut, self.address_block_id, "dpacc");

        let bc = get_bc_for!(dut, reg, "a")?;
        println!("Addr {:?}", (transaction.addr()? & 0xF) >> 2);
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(transaction.data);

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = reg.bits(dut);
        self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;

        // // Update the IR for access type
        // //self.update_ir(&dut, &services, self.dpacc_select)?;

        // // Write the header and data
        // let mut trans = transaction.clone();
        // //self.prep_transaction(&mut trans, false)?;
        // jtag.write_dr(&dut, &trans)?;

        // let mut t = transaction.clone();
        // t.prepend_data(BigUint::from(0 as u8), 3)?;
        // t.bit_enable = BigUint::from(t.bit_enable & BigUint::from(0x7 as u8));
        // let jtag_service = services.get_service(self.jtag_id)?;
        // let jtag = jtag_service.as_jtag()?;
        // jtag.write_dr(&dut, &t)?;


        // TEST.close(n_id)?;
        Ok(())
    }

    // pub fn verify_dp(&self, dut: &Dut, services: &Services, transaction: Transaction) -> Result<()> {
    //     let jtag_service = services.get_service(self.jtag_id)?;
    //     let jtag = jtag_service.as_jtag()?;
    //     let mut trans = node!(
    //         ArmDebugJTAGVerifyDP,
    //         self.id,
    //         transaction.clone(),
    //         None
    //     );
    //     let n_id = TEST.push_and_open(trans.clone());

    //     // Update the IR for access type
    //     //self.update_ir(&dut, &services, self.dpacc_select)?;

    //     // Write the header and data
    //     let mut trans = transaction.clone();
    //     //self.prep_transaction(&mut trans, true)?;
    //     jtag.verify_dr(&dut, &trans)?;

    //     TEST.close(n_id)?;
    //     Ok(())
    // }
    pub fn verify_dp(&self, dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction, update_dpacc: bool) -> Result<()> {
        let reg = get_reg!(dut, self.address_block_id, "dpacc");

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = get_bc_for!(dut, reg, "a")?;
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(1 as u8));

        let bc = reg.bits(dut);
        if update_dpacc {
            self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;
        } else {
            self.update_ir(&dut, &services, self.dpacc_select as u32)?;
        }

        let mut t = transaction.clone();
        t.prepend_data(BigUint::from(0 as u8), 3)?;
        t.bit_enable = t.bit_enable & BigUint::from(0x7_FFFF_FFF8 as u64);
        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        jtag.verify_dr(&dut, &t)?;
        Ok(())
    }

    pub fn write_ap(&self, dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction) -> Result<()> {
        let reg = get_reg!(dut, self.address_block_id, "apacc");

        let bc = get_bc_for!(dut, reg, "a")?;
        println!("Addr {:?}", (transaction.addr()? & 0xF) >> 2);
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(transaction.data);

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = reg.bits(dut);
        self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;

        Ok(())
    }

    pub fn verify_ap(&self,  dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction, update_apacc: bool) -> Result<()> {
        let reg = get_reg!(dut, self.address_block_id, "apacc");

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = get_bc_for!(dut, reg, "a")?;
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(1 as u8));

        let bc = reg.bits(dut);
        if update_apacc {
            self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;
        } else {
            self.update_ir(&dut, &services, self.apacc_select as u32)?;
        }

        let mut t = transaction.clone();
        t.prepend_data(BigUint::from(0 as u8), 3)?;
        t.bit_enable = t.bit_enable & BigUint::from(0x7_FFFF_FFF8 as u64);
        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        jtag.verify_dr(&dut, &t)?;
        Ok(())
    }

    pub fn write_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, trans: Transaction) -> Result<()> {
        //let trans = bc.to_write_transaction(&dut)?;
        {
            let jtag_service = services.get_service(self.jtag_id)?;
            let jtag = jtag_service.as_jtag()?;
            jtag.reset(&dut)?;
        }
        self.update_ir(&dut, &services, trans.addr()? as u32)?;
        // self.write_dp(&dut, &services, trans)?;
        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        jtag.write_dr(&dut, &trans)?;
        Ok(())
    }

    pub fn verify_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, transaction: Transaction) -> Result<()> {
        {
            let jtag_service = services.get_service(self.jtag_id)?;
            let jtag = jtag_service.as_jtag()?;
            jtag.reset(&dut)?;
        }

        //let trans = bc.to_verify_transaction(None, true, &dut)?;
        self.update_ir(&dut, &services, transaction.addr()? as u32)?;
        // self.verify_dp(&dut, &services, trans)?;
        // Ok(())

        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        let mut trans = node!(
            ArmDebugJTAGVerifyDP,
            self.id,
            transaction.clone(),
            None
        );
        let n_id = TEST.push_and_open(trans.clone());

        // Update the IR for access type
        //self.update_ir(&dut, &services, self.dpacc_select)?;

        // Write the header and data
        let mut trans = transaction.clone();
        //self.prep_transaction(&mut trans, true)?;
        jtag.verify_dr(&dut, &trans)?;

        TEST.close(n_id)?;
        Ok(())

    }
}