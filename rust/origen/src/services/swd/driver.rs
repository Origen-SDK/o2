use crate::generator::ast::*;
use crate::{generate_even_parity};
use super::Acknowledgements;
use num::ToPrimitive;
use crate::testers::vector_based::api::*;
use super::service::Service;
use crate::{Result, Error};

impl Service {
    #[allow(non_snake_case)]
    fn drive_header(&self, ap_dp_addr: u32, ap_access: bool, verify: bool, nodes: &mut Vec<Node>) -> Result<()> {
        let A = (ap_dp_addr & 0xF) >> 2;
        nodes.push(comment(&format!(
            "SWD: Header: host -> target (A: {:X}, ap_access: {}, verify: {}) ",
            A,
            ap_access as u8,
            verify as u8
        ))?);
        nodes.append(&mut drive_high(&self.swdclk_id, self.swdclk_grp_id)?); // Ensure swdclk is running
        nodes.append(&mut drive_high(&self.swdio_id, self.swdio_grp_id)?); // Start bit
        nodes.push(cycle(true)?);
        if ap_access {
            nodes.append(&mut drive_high(&self.swdio_id, self.swdio_grp_id)?); // Indicate AP access
            nodes.push(cycle(true)?);
        } else {
            nodes.append(&mut drive_low(&self.swdio_id, self.swdio_grp_id)?); // Indicate DP access
            nodes.push(cycle(true)?);
        }
        if verify {
            nodes.append(&mut drive_high(&self.swdio_id, self.swdio_grp_id)?); // Indicate read operation
            nodes.push(cycle(true)?);
        } else {
            nodes.append(&mut drive_low(&self.swdio_id, self.swdio_grp_id)?); // Indicate write operation
            nodes.push(cycle(true)?);
        }
        nodes.append(&mut push_data(&self.swdio_grp, &A, 2, true)?); // Drive the AP target
        nodes.append(&mut drive(&self.swdio_id, generate_even_parity!(
            A + ((ap_access as u32) << 3) + ((verify as u32) << 2)
        ) as bool, self.swdio_grp_id)?); // Drive the parity bit
        nodes.push(cycle(true)?);
        nodes.append(&mut drive_low(&self.swdio_id, self.swdio_grp_id)?); // Stop bit
        nodes.push(cycle(true)?);
        nodes.append(&mut drive_high(&self.swdio_id, self.swdio_grp_id)?); // Park bit
        nodes.push(cycle(true)?);
        nodes.append(&mut highz(&self.swdio_id, self.swdio_grp_id)?);
        nodes.push(repeat(self.trn.into(), true)?); // TRN
        Ok(())
    }

    fn verify_ack(&self, ack: &Acknowledgements, nodes: &mut Vec<Node>) -> Result<()> {
        match ack {
            Acknowledgements::Ok => {
                nodes.push(comment("SWD: Acknowledge Ok: target -> host")?);
                nodes.append(&mut verify_high(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
            },
            Acknowledgements::Wait => {
                nodes.push(comment("SWD: Acknowledge Wait: target -> host")?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_high(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
            },
            Acknowledgements::Fault => {
                nodes.push(comment("SWD: Acknowledge Fault: target -> host")?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_low(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.append(&mut verify_high(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
            },
            Acknowledgements::None => {
                nodes.push(comment("SWD: Do not check acknowledgement")?);
                nodes.append(&mut highz(&self.swdio_id, self.swdio_grp_id)?);
                nodes.push(cycle(true)?);
                nodes.push(repeat(3, true)?);
            }
        }
        Ok(())
    }

    fn drive_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>, overlays: &Option<num_bigint::BigUint>, overlay_str: &Option<String>) -> Result<()> {
        nodes.append(&mut highz(&self.swdio_id, self.swdio_grp_id)?);
        nodes.push(repeat(self.trn.into(), true)?); // TRN
        nodes.push(comment("SWD: Drive data")?);
        nodes.append(&mut drive_data(
            &self.swdio_grp,
            data,
            32,
            true,
            overlays.as_ref(),
            overlay_str.as_ref()
        )?);
        nodes.push(comment("SWD: Drive data's parity bit")?);
        nodes.append(&mut drive(
            &self.swdio_id,
            generate_even_parity!(data.to_u32().unwrap()).into(),
            self.swdio_grp_id
        )?); // Drive the AP target's parity
        nodes.push(cycle(true)?);
        Ok(())
    }

    fn verify_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>, parity: &Option<bool>, verifies: &Option<num_bigint::BigUint>, captures: &Option<num_bigint::BigUint>) -> Result<()> {
        nodes.push(comment("SWD: Verify data")?);
        nodes.append(&mut read_data(
            &self.swdio_grp,
            data,
            32,
            true,
            verifies.as_ref(),
            captures.as_ref(),
        )?);
        if let Some(p) = parity {
            nodes.push(comment(&format!("SWD: Expecting parity bit of {}", *p as u8))?);
            nodes.append(&mut verify(&self.swdio_id, *p, self.swdio_grp_id)?);
            nodes.push(cycle(true)?);
        } else {
            nodes.push(comment("SWD: Ignoring parity bit on SWD READ operation")?);
            nodes.append(&mut highz(&self.swdio_id, self.swdio_grp_id)?);
            nodes.push(cycle(true)?);
        }
        nodes.push(repeat(self.trn.into(), true)?); // TRN
        Ok(())
    }

    fn close_swd(&self, _nodes: &mut Vec<Node>) -> Result<()> {
        // nodes.push(comment("  SWD: Disable SWDCLK and SWDIO")?);
        // nodes.push(set_highz!(&self.swdio));
        // nodes.push(set_drive_low!(&self.swdclk));
        Ok(())
    }

    pub fn process_transaction(&self, node: &mut Node) -> Result<()> {
        match &node.attrs {
            Attrs::SWDWriteAP(_swd_id, transaction, ack, _metadata) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Write AP - AP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ))?);
                self.drive_header(transaction.addr()? as u32, true, false, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(
                    &transaction.data,
                    &mut nodes,
                    &transaction.overlay_enable,
                    &transaction.overlay_string
                )?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
                Ok(())
            },
            Attrs::SWDVerifyAP(_swd_id, transaction, ack, parity_compare, _metadata) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Verify AP - AP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ))?);
                self.drive_header(
                    transaction.addr()? as u32,
                    true,
                    true,
                    &mut nodes
                )?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(
                    &transaction.data,
                    &mut nodes,
                    &parity_compare,
                    &transaction.verify_enable,
                    &transaction.capture_enable)?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
                Ok(())
            },
            Attrs::SWDWriteDP(_swd_id, transaction, ack, _metadata) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Write DP - DP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ))?);
                self.drive_header(transaction.addr()? as u32, false, false, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(
                    &transaction.data,
                    &mut nodes,
                    &transaction.overlay_enable,
                    &transaction.overlay_string
                )?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
                Ok(())
            },
            Attrs::SWDVerifyDP(_swd_id, transaction, ack, parity_compare, _metadata) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Verify DP - DP: {}, Data: 0x{:X}",
                    transaction.addr()?,
                    transaction.data,
                ))?);
                self.drive_header(transaction.addr()? as u32, false, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(
                    &transaction.data,
                    &mut nodes,
                    &parity_compare,
                    &transaction.verify_enable,
                    &transaction.capture_enable,
                )?;
                self.close_swd(&mut nodes)?;
                node.add_children(nodes);
                Ok(())
            },
            _ => Err(Error::new(&format!("Unexpected node in SWD driver: {:?}", node)))
        }
    }

    pub fn line_reset(&self) -> Result<Node> {
        let mut n = node!(SWDLineReset);
        n.add_child(comment("SWD: Line Reset")?);
        n.add_children(drive_high(&self.swdclk_id, self.swdclk_grp_id)?);
        n.add_children(drive_high(&self.swdio_id, self.swdio_grp_id)?);
        n.add_child(repeat(50, true)?);
        n.add_children(highz(&self.swdio_id, self.swdio_grp_id)?);
        n.add_child(repeat(2, true)?);
        Ok(n)
    }
}