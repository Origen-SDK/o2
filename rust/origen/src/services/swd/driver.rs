use crate::generator::ast::*;
use super::Acknowledgements;
use crate::testers::vector_based::api::*;
use super::service::Service;
use crate::{Result, Error, Transaction, TEST};
use crate::testers::api::ControllerAPI;
use crate::utility::num_helpers::NumHelpers;
use crate::core::model::pins::PinCollection;

impl ControllerAPI for Service {
    fn name(&self) -> String {
        "SWD".to_string()
    }
}

impl Service {
    #[allow(non_snake_case)]
    fn drive_header(&self, swdclk: &PinCollection, swdio: &PinCollection, ap_dp_addr: u32, ap_access: bool, verify: bool) -> Result<()> {
        let A = (ap_dp_addr & 0xF) >> 2;
        self.comment(&format!(
            "Header: host -> target (A: {:X}, ap_access: {}, verify: {}) ",
            A,
            ap_access as u8,
            verify as u8
        ));
        swdclk.drive_high();
        swdio.drive_high().cycle();
        swdio.drive(ap_access).cycle();
        swdio.drive(verify).cycle();
        swdio.push_transaction(&crate::Transaction::new_write(A.into(), 2)?)?;
        swdio.drive(
            (A + ((ap_access as u32) << 3) + ((verify as u32) << 2)).even_parity()
        ).cycle();
        swdio.drive_low().cycle();
        swdio.drive_high().cycle();
        swdio.highz().repeat(self.trn.into());
        Ok(())
    }

    fn verify_ack(&self, swdio: &PinCollection, ack: &Acknowledgements) -> Result<()> {
        match ack {
            Acknowledgements::Ok => {
                self.comment("Acknowledge Ok: target -> host");
                swdio.verify_high().cycle();
                swdio.verify_low().repeat(2);
            },
            Acknowledgements::Wait => {
                self.comment("Acknowledge Wait: target -> host");
                swdio.verify_low().cycle();
                swdio.verify_high().cycle();
                swdio.verify_low().cycle();
            },
            Acknowledgements::Fault => {
                self.comment("Acknowledge Fault: target -> host");
                swdio.verify_low().repeat(2);
                swdio.verify_high().cycle();
            },
            Acknowledgements::None => {
                self.comment("Do not check acknowledgement");
                swdio.highz().repeat(3);
            }
        }
        Ok(())
    }

    fn drive_data(&self, swdio: &PinCollection, trans: &Transaction) -> Result<()> {
        swdio.highz().repeat(self.trn.into());
        self.comment("Drive data");
        swdio.push_transaction(trans)?;
        self.comment("Drive data's parity bit");
        swdio.drive(trans.even_parity()).cycle();
        Ok(())
    }

    fn verify_data(&self, swdio: &PinCollection, trans: &Transaction, parity: &Option<bool>) -> Result<()> {
        self.comment("Verify data");
        swdio.push_transaction(trans)?;
        if let Some(p) = parity {
            self.comment(&format!("Expecting parity bit of {}", *p as u8));
            swdio.verify(*p).cycle();
            swdio.highz();
        } else {
            self.comment("Ignoring parity bit on SWD READ operation");
            swdio.highz().cycle();
        }
        repeat(self.trn.into());
        Ok(())
    }

    fn close_swd(&self, _nodes: &mut Vec<Node>) -> Result<()> {
        Ok(())
    }

    pub fn process_transaction(&self, dut: &crate::Dut, node: &mut Node) -> Result<()> {
        let swdclk = PinCollection::from_group(dut, &self.swdclk.0, self.swdclk.1)?;
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        match &node.attrs {
            Attrs::SWDWriteAP(_swd_id, transaction, ack, _metadata) => {
                let mut nodes = vec!();
                self.comment(&format!(
                    "Write AP - AP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ));
                self.drive_header(&swdclk, &swdio, transaction.addr()? as u32, true, false)?;
                self.verify_ack(&swdio, ack)?;
                self.drive_data(&swdio, transaction)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
            },
            Attrs::SWDVerifyAP(_swd_id, transaction, ack, parity_compare, _metadata) => {
                let mut nodes = vec!();
                self.comment(&format!(
                    "Verify AP - AP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ));
                self.drive_header(
                    &swdclk,
                    &swdio,
                    transaction.addr()? as u32,
                    true,
                    true,
                )?;
                self.verify_ack(&swdio, ack)?;
                self.verify_data(&swdio, transaction, &parity_compare)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
            },
            Attrs::SWDWriteDP(_swd_id, transaction, ack, _metadata) => {
                let mut nodes = vec!();
                self.comment(&format!(
                    "Write DP - DP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ));
                self.drive_header(&swdclk, &swdio, transaction.addr()? as u32, false, false)?;
                self.verify_ack(&swdio, ack)?;
                self.drive_data(&swdio, transaction)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
            },
            Attrs::SWDVerifyDP(_swd_id, transaction, ack, parity_compare, _metadata) => {
                let mut nodes = vec!();
                self.comment(&format!(
                    "Verify DP - DP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ));
                self.drive_header(&swdclk, &swdio, transaction.addr()? as u32, false, true)?;
                self.verify_ack(&swdio, ack)?;
                self.verify_data(&swdio, transaction, &parity_compare)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
            },
            _ => return Err(Error::new(&format!("Unexpected node in SWD driver: {:?}", node)))
        }
        Ok(())
    }

    pub fn line_reset(&self, dut: &crate::Dut) -> Result<usize> {
        let n_id = TEST.push_and_open(node!(SWDLineReset));
        let swdclk = PinCollection::from_group(dut, &self.swdclk.0, self.swdclk.1)?;
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        self.comment("Line Reset");
        swdclk.drive_high();
        swdio.drive_high();
        repeat(50);
        swdio.highz().repeat(2);
        TEST.close(n_id)?;
        Ok(n_id)
    }
}