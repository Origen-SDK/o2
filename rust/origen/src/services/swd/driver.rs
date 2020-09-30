use crate::generator::ast::*;
use super::Acknowledgements;
use crate::testers::vector_based::api::*;
use super::service::Service;
use crate::{Result, Error, Transaction, TEST};
use crate::testers::api::ControllerAPI;
use crate::utility::num_helpers::NumHelpers;

impl ControllerAPI for Service {
    fn name(&self) -> String {
        "SWD".to_string()
    }
}

impl Service {
    #[allow(non_snake_case)]
    fn drive_header(&self, ap_dp_addr: u32, ap_access: bool, verify: bool) -> Result<()> {
        let A = (ap_dp_addr & 0xF) >> 2;
        self.comment(&format!(
            "Header: host -> target (A: {:X}, ap_access: {}, verify: {}) ",
            A,
            ap_access as u8,
            verify as u8
        ));
        self.swdclk.drive_high();
        self.swdio.drive_high().cycle();
        self.swdio.drive(ap_access).cycle();
        self.swdio.drive(verify).cycle();
        self.swdio.push_transaction(&crate::Transaction::new_write(A.into(), 2)?)?;
        self.swdio.drive(
            (A + ((ap_access as u32) << 3) + ((verify as u32) << 2)).even_parity()
        ).cycle();
        self.swdio.drive_low().cycle();
        self.swdio.drive_high().cycle();
        self.swdio.highz().repeat(self.trn.into());
        Ok(())
    }

    fn verify_ack(&self, ack: &Acknowledgements) -> Result<()> {
        match ack {
            Acknowledgements::Ok => {
                self.comment("Acknowledge Ok: target -> host");
                self.swdio.verify_high().cycle();
                self.swdio.verify_low().repeat(2);
            },
            Acknowledgements::Wait => {
                self.comment("Acknowledge Wait: target -> host");
                self.swdio.verify_low().cycle();
                self.swdio.verify_high().cycle();
                self.swdio.verify_low().cycle();
            },
            Acknowledgements::Fault => {
                self.comment("Acknowledge Fault: target -> host");
                self.swdio.verify_low().repeat(2);
                self.swdio.verify_high().cycle();
            },
            Acknowledgements::None => {
                self.comment("Do not check acknowledgement");
                self.swdio.highz().repeat(3);
            }
        }
        Ok(())
    }

    fn drive_data(&self, trans: &Transaction) -> Result<()> {
        self.swdio.highz().repeat(self.trn.into());
        self.comment("Drive data");
        self.swdio.push_transaction(trans)?;
        self.comment("Drive data's parity bit");
        self.swdio.drive(trans.even_parity()).cycle();
        Ok(())
    }

    fn verify_data(&self, trans: &Transaction, parity: &Option<bool>) -> Result<()> {
        self.comment("Verify data");
        self.swdio.push_transaction(trans)?;
        if let Some(p) = parity {
            self.comment(&format!("Expecting parity bit of {}", *p as u8));
            self.swdio.verify(*p).cycle();
            self.swdio.highz();
        } else {
            self.comment("Ignoring parity bit on SWD READ operation");
            self.swdio.highz().cycle();
        }
        repeat(self.trn.into());
        Ok(())
    }

    fn close_swd(&self, _nodes: &mut Vec<Node>) -> Result<()> {
        Ok(())
    }

    pub fn process_transaction(&self, dut: &crate::Dut, node: &mut Node) -> Result<()> {
        match &node.attrs {
            Attrs::SWDWriteAP(_swd_id, transaction, ack, _metadata) => {
                let mut nodes = vec!();
                self.comment(&format!(
                    "Write AP - AP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ));
                self.drive_header(transaction.addr()? as u32, true, false)?;
                self.verify_ack(ack)?;
                self.drive_data(transaction)?;
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
                    transaction.addr()? as u32,
                    true,
                    true,
                )?;
                self.verify_ack(ack)?;
                self.verify_data(transaction, &parity_compare)?;
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
                self.drive_header(transaction.addr()? as u32, false, false)?;
                self.verify_ack(ack)?;
                self.drive_data(transaction)?;
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
                self.drive_header(transaction.addr()? as u32, false, true)?;
                self.verify_ack(ack)?;
                self.verify_data(transaction, &parity_compare)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
            },
            _ => return Err(Error::new(&format!("Unexpected node in SWD driver: {:?}", node)))
        }
        self.update_actions(dut)
    }

    pub fn line_reset(&self, dut: &crate::Dut) -> Result<usize> {
        let n_id = TEST.push_and_open(node!(SWDLineReset));
        self.comment("Line Reset");
        self.swdclk.drive_high();
        self.swdio.drive_high();
        repeat(50);
        self.swdio.highz().repeat(2);
        TEST.close(n_id)?;
        self.update_actions(dut)?;
        Ok(n_id)
    }
}