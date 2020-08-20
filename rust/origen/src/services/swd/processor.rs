use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::{set_drive_high, drive_high, drive_low, drive_pin, cycle, drive_data, verify_high, verify_low, set_highz, set_drive_low, generate_even_parity, comment, verify_pin, highz, verify_data};
use super::Acknowledgements;
use num::ToPrimitive;

// Transforms SWD nodes into pin states transactions.
// Target use case for non-protocol-aware, vector-based testers.
pub struct SWDToPinStates {
    swdio: String,
    swdclk: String,
    trn: u32,
}

impl SWDToPinStates {
    pub fn run(node: &Node, swdclk: &str, swdio: &str, trn: Option<u32>) -> Result<Node> {
        let mut p;
        {
            let dut = crate::DUT.lock().unwrap();
            p = Self {
                swdio: {
                    dut._resolve_group_to_physical_pins(0, swdio)?.first().unwrap().name.to_string()
                },
                swdclk: {
                    dut._resolve_group_to_physical_pins(0, swdclk)?.first().unwrap().name.to_string()
                },
                trn: trn.unwrap_or(1)
            };
        }
        Ok(node.process(&mut p)?.unwrap())
    }

    fn drive_header(&self, ap_dp_addr: u32, ap_access: bool, read: bool, nodes: &mut Vec<Node>) -> Result<()> {
        nodes.push(comment!("  SWD: Header: host -> target"));
        nodes.append(&mut drive_high!(&self.swdclk)); // Ensure SWDCLK is running
        nodes.append(&mut drive_high!(&self.swdio)); // Start bit
        if ap_access {
            nodes.append(&mut drive_high!(&self.swdio)); // Indicate AP access
        } else {
            nodes.append(&mut drive_low!(&self.swdio)); // Indicate DP access
        }
        if read {
            nodes.append(&mut drive_high!(&self.swdio)); // Indicate read operation
        } else {
            nodes.append(&mut drive_low!(&self.swdio)); // Indicate write operation
        }
        nodes.append(&mut drive_data!(vec!(&self.swdio), ap_dp_addr, 3, None)); // Drive the AP target
        nodes.append(&mut drive_pin!(&self.swdio, generate_even_parity!(ap_dp_addr))); // Drive the AP target's parity
        nodes.append(&mut drive_low!(&self.swdio)); // Stop bit
        nodes.append(&mut drive_high!(&self.swdio)); // Park bit
        nodes.push(cycle!(self.trn)); // TRN
        Ok(())
    }

    fn verify_ack(&self, ack: &Acknowledgements, nodes: &mut Vec<Node>) -> Result<()> {
        match ack {
            Acknowledgements::Ok => {
                nodes.push(comment!("  SWD: Acknowledge Ok: target -> host"));
                nodes.append(&mut verify_high!(&self.swdio));
                nodes.append(&mut verify_low!(&self.swdio));
                nodes.append(&mut verify_low!(&self.swdio));
            },
            Acknowledgements::Wait => {
                nodes.push(comment!("  SWD: Acknowledge Wait: target -> host"));
                nodes.append(&mut verify_low!(&self.swdio));
                nodes.append(&mut verify_high!(&self.swdio));
                nodes.append(&mut verify_low!(&self.swdio));
            },
            Acknowledgements::Fault => {
                nodes.push(comment!("  SWD: Acknowledge Fault: target -> host"));
                nodes.append(&mut verify_low!(&self.swdio));
                nodes.append(&mut verify_low!(&self.swdio));
                nodes.append(&mut verify_high!(&self.swdio));
            },
            Acknowledgements::None => {
                nodes.push(comment!("  SWD: Do not check acknowledgement"));
                nodes.push(set_highz!(&self.swdio));
                nodes.push(cycle!(3));
            }
        }
        Ok(())
    }

    fn drive_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>) -> Result<()> {
        nodes.push(cycle!(self.trn)); // TRN
        nodes.push(comment!("  SWD: Drive data"));
        nodes.append(&mut drive_data!(vec!(&self.swdio), data.to_u32().unwrap(), 32, None)); // Drive the AP target
        nodes.push(comment!("  SWD: Drive data's parity bit"));
        nodes.append(&mut drive_pin!(&self.swdio, generate_even_parity!(data.to_u32().unwrap()))); // Drive the AP target's parity
        Ok(())
    }

    fn verify_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>, parity: &Option<bool>) -> Result<()> {
        nodes.push(comment!("  SWD: Verify data"));
        nodes.append(&mut verify_data!(
            vec!(&self.swdio),
            data.to_u32().unwrap(), 32, None
        ));
        if let Some(p) = parity {
            nodes.push(comment!(&format!("  SWD: Expecting parity bit of {}", *p as u8)));
            nodes.append(&mut verify_pin!(&self.swdio, *p));
        } else {
            nodes.push(comment!("  SWD: Ignoring parity bit on SWD READ operation"));
            nodes.append(&mut highz!(&self.swdio));
        }
        nodes.push(cycle!(self.trn)); // TRN
        Ok(())
    }

    fn close_swd(&self, nodes: &mut Vec<Node>) -> Result<()> {
        nodes.push(comment!("  SWD: Disable SWDCLK and SWDIO"));
        nodes.push(set_highz!(&self.swdio));
        nodes.push(set_drive_low!(&self.swdclk));
        Ok(())
    }
}

// https://static.docs.arm.com/ihi0031/c/IHI0031C_debug_interface_as.pdf
impl Processor for SWDToPinStates {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::SWDWriteAP(data, ap_addr, ack) => {
                let mut nodes = vec!();
                nodes.push(comment!(&format!(
                    "SWD: Write AP - AP: {}, Data: 0x{:X}",
                    ap_addr,
                    data,
                )));
                self.drive_header(*ap_addr, true, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(data, &mut nodes)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            // Attrs::SWDBufferedWriteAP => {
            //     // ...
            // },
            Attrs::SWDVerifyAP(data, ap_addr, ack, parity) => {
                let mut nodes = vec!();
                nodes.push(comment!(&format!(
                    "SWD: Verify AP - AP: {}, Data: 0x{:X}",
                    ap_addr,
                    data,
                )));
                self.drive_header(*ap_addr, true, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(data, &mut nodes, &parity)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            Attrs::SWDWriteDP(data, dp_addr, ack) => {
                let mut nodes = vec!();
                nodes.push(comment!(&format!(
                    "SWD: Write DP - DP: {}, Data: 0x{:X}",
                    dp_addr,
                    data,
                )));
                self.drive_header(*dp_addr, false, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(data, &mut nodes)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            // Attrs::SWDBufferedWriteDP => {
            //     // ...
            // },
            Attrs::SWDVerifyDP(data, dp_addr, ack, parity) => {
                let mut nodes = vec!();
                nodes.push(comment!(&format!(
                    "SWD: Verify DP - DP: {}, Data: 0x{:X}",
                    dp_addr,
                    data,
                )));
                self.drive_header(*dp_addr, false, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(data, &mut nodes, &parity)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            Attrs::SWDLineReset => {
                let mut nodes = vec!();
                nodes.push(comment!("SWD: Line Reset"));
                nodes.push(set_drive_high!(self.swdclk));
                nodes.push(set_drive_high!(self.swdio));
                nodes.push(cycle!(50));
                nodes.push(set_highz!(self.swdio));
                nodes.push(cycle!(2));
                Ok(Return::Inline(nodes))
            },
            _ => Ok(Return::ProcessChildren),
        }
    }
}
