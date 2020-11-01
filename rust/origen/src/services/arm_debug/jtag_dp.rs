use crate::{Result, Error, TEST, Dut, Services,
    add_reg_32bit, add_reg, field, some_hard_reset_val, get_bc_for, get_reg
};
use num_bigint::BigUint;
use std::sync::MutexGuard;
use super::super::super::services::Service;
use crate::precludes::controller::*;

#[derive(Clone, Debug)]
pub struct JtagDP {
    pub id: usize,
    model_id: usize,
    memory_map_id: usize,
    address_block_id: usize,
    arm_debug_id: usize,
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
        let jtag_id;
        {
            let arm_debug = services.get_as_arm_debug(arm_debug_id)?;
            if let Some(j_id) = arm_debug.jtag_id {
               jtag_id = j_id;
            } else {
                return Err(Error::new("Arm Debug's JTAG DP requires a JTAG service. No JTAG service was given"));
            }
        }
        services.push_service(Service::ArmDebugJtagDP(
            Self {
                id: id,
                model_id: model_id,
                memory_map_id: memory_map_id,
                address_block_id: ab_id,
                arm_debug_id: arm_debug_id,
                jtag_id: jtag_id,
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
        let n_id = TEST.push_and_open(node!(JTAGDPWriteDP, self.id, transaction.clone(), None));

        let reg = get_reg!(dut, self.address_block_id, "dpacc");

        let bc = get_bc_for!(dut, reg, "a")?;
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(transaction.data);

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = reg.bits(dut);
        self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;

        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_dp(&self, dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction, write_dpacc: bool) -> Result<()> {
        let n_id = TEST.push_and_open(node!(JTAGDPVerifyDP, self.id, transaction.clone(), None));
        if write_dpacc {
            let reg = get_reg!(dut, self.address_block_id, "dpacc");

            let bc = get_bc_for!(dut, reg, "data")?;
            bc.set_data(BigUint::from(0 as u8));
    
            let bc = get_bc_for!(dut, reg, "a")?;
            bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));
    
            let bc = get_bc_for!(dut, reg, "rnw")?;
            bc.set_data(BigUint::from(1 as u8));
    
            let bc = reg.bits(dut);
    
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
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write_ap(&self, dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction) -> Result<()> {
        let n_id = TEST.push_and_open(node!(JTAGDPWriteAP, self.id, transaction.clone(), None));
        let reg = get_reg!(dut, self.address_block_id, "apacc");

        let bc = get_bc_for!(dut, reg, "a")?;
        bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));

        let bc = get_bc_for!(dut, reg, "data")?;
        bc.set_data(transaction.data);

        let bc = get_bc_for!(dut, reg, "rnw")?;
        bc.set_data(BigUint::from(0 as u8));

        let bc = reg.bits(dut);
        self.write_register(dut, services, bc.to_write_transaction(&dut)?)?;

        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_ap(&self,  dut: &MutexGuard<Dut>, services: &Services, transaction: Transaction, write_apacc: bool) -> Result<()> {
        let n_id = TEST.push_and_open(node!(JTAGDPVerifyAP, self.id, transaction.clone(), None));
        if write_apacc {
            let reg = get_reg!(dut, self.address_block_id, "apacc");

            let bc = get_bc_for!(dut, reg, "data")?;
            bc.set_data(BigUint::from(0 as u8));
    
            let bc = get_bc_for!(dut, reg, "a")?;
            bc.set_data(BigUint::from((transaction.addr()? & 0xF) >> 2 as u8));
    
            let bc = get_bc_for!(dut, reg, "rnw")?;
            bc.set_data(BigUint::from(1 as u8));
    
            let bc = reg.bits(dut);
    
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
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, trans: Transaction) -> Result<()> {
        let n_id = TEST.push_and_open(trans.as_write_node()?);
        {
            let jtag_service = services.get_service(self.jtag_id)?;
            let jtag = jtag_service.as_jtag()?;
            jtag.reset(&dut)?;
        }
        self.update_ir(&dut, &services, trans.addr()? as u32)?;

        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;
        jtag.write_dr(&dut, &trans)?;

        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_register(&self, dut: &MutexGuard<Dut>, services: &crate::Services, transaction: Transaction) -> Result<()> {
        let n_id = TEST.push_and_open(transaction.as_verify_node()?);
        {
            let jtag_service = services.get_service(self.jtag_id)?;
            let jtag = jtag_service.as_jtag()?;
            jtag.reset(&dut)?;
        }
        self.update_ir(&dut, &services, transaction.addr()? as u32)?;

        let jtag_service = services.get_service(self.jtag_id)?;
        let jtag = jtag_service.as_jtag()?;

        // Write the header and data
        let trans = transaction.clone();
        jtag.verify_dr(&dut, &trans)?;

        TEST.close(n_id)?;
        Ok(())
    }
}