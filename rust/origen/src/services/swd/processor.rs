use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::{generate_even_parity};
use super::Acknowledgements;
use num::ToPrimitive;
use crate::testers::vector_based::api::*;

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
        nodes.push(comment("  SWD: Header: host -> target")?);
        nodes.append(&mut drive_pin_high(&self.swdio)?); // Start bit
        if ap_access {
            nodes.append(&mut drive_pin_high(&self.swdio)?); // Indicate AP access
        } else {
            nodes.append(&mut drive_pin_low(&self.swdio)?); // Indicate DP access
        }
        if read {
            nodes.append(&mut drive_pin_high(&self.swdio)?); // Indicate read operation
        } else {
            nodes.append(&mut drive_pin_low(&self.swdio)?); // Indicate write operation
        }
        nodes.append(&mut push_data(vec!(&self.swdio), &ap_dp_addr, 2, true)?); // Drive the AP target
        nodes.append(&mut drive_pin(&self.swdio, generate_even_parity!(
            ap_dp_addr + ((ap_access as u32) << 3) + ((read as u32) << 2)
        ) as bool)?); // Drive the parity bit
        nodes.append(&mut drive_pin_low(&self.swdio)?); // Stop bit
        nodes.append(&mut drive_pin_high(&self.swdio)?); // Park bit
        nodes.push(set_pin_highz(&self.swdio)?);
        nodes.push(repeat(self.trn.into())?); // TRN
        Ok(())
    }

    fn verify_ack(&self, ack: &Acknowledgements, nodes: &mut Vec<Node>) -> Result<()> {
        match ack {
            Acknowledgements::Ok => {
                nodes.push(comment("  SWD: Acknowledge Ok: target -> host")?);
                nodes.append(&mut verify_pin_high(&self.swdio)?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
            },
            Acknowledgements::Wait => {
                nodes.push(comment("  SWD: Acknowledge Wait: target -> host")?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
                nodes.append(&mut verify_pin_high(&self.swdio)?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
            },
            Acknowledgements::Fault => {
                nodes.push(comment("  SWD: Acknowledge Fault: target -> host")?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
                nodes.append(&mut verify_pin_low(&self.swdio)?);
                nodes.append(&mut verify_pin_high(&self.swdio)?);
            },
            Acknowledgements::None => {
                nodes.push(comment("  SWD: Do not check acknowledgement")?);
                nodes.push(set_pin_highz(&self.swdio)?);
                nodes.push(repeat(3)?);
            }
        }
        Ok(())
    }

    fn drive_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>, overlays: &Option<num_bigint::BigUint>, overlay_str: &Option<String>) -> Result<()> {
        nodes.push(repeat(self.trn.into())?); // TRN
        nodes.push(comment("  SWD: Drive data")?);
        nodes.append(&mut drive_data(
            vec!(&self.swdio),
            data,
            32,
            true,
            overlays.as_ref(),
            overlay_str.as_ref()
        )?);
        nodes.push(comment("  SWD: Drive data's parity bit")?);
        nodes.append(&mut drive_pin(&self.swdio, generate_even_parity!(data.to_u32().unwrap()).into())?); // Drive the AP target's parity
        Ok(())
    }

    fn verify_data(&self, data: &num::BigUint, nodes: &mut Vec<Node>, parity: &Option<bool>, verifies: &Option<num_bigint::BigUint>, captures: &Option<num_bigint::BigUint>) -> Result<()> {
        nodes.push(comment("  SWD: Verify data")?);
        nodes.append(&mut read_data(
            vec!(&self.swdio),
            data,
            32,
            true,
            verifies.as_ref(),
            captures.as_ref(),
        )?);
        if let Some(p) = parity {
            nodes.push(comment(&format!("  SWD: Expecting parity bit of {}", *p as u8))?);
            nodes.append(&mut verify_pin(&self.swdio, *p)?);
        } else {
            nodes.push(comment("  SWD: Ignoring parity bit on SWD READ operation")?);
            nodes.append(&mut highz_pin(&self.swdio)?);
        }
        nodes.push(repeat(self.trn.into())?); // TRN
        Ok(())
    }

    fn close_swd(&self, nodes: &mut Vec<Node>) -> Result<()> {
        // nodes.push(comment("  SWD: Disable SWDCLK and SWDIO")?);
        // nodes.push(set_highz!(&self.swdio));
        // nodes.push(set_drive_low!(&self.swdclk));
        Ok(())
    }
}

// https://static.docs.arm.com/ihi0031/c/IHI0031C_debug_interface_as.pdf
impl Processor for SWDToPinStates {
    // The bitfield in the above doc is literally "A", so allow that here.
    #[allow(non_snake_case)]
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::SWDWriteAP(data, A, ack, overlays, overlay_str) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Write AP - AP: {}, Data: 0x{:X}",
                    A,
                    data,
                ))?);
                self.drive_header(*A, true, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(data, &mut nodes, overlays, overlay_str)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            // Attrs::SWDBufferedWriteAP => {
            //     // ...
            // },
            Attrs::SWDVerifyAP(data, A, ack, parity_compare, verifies, captures, overlay, overlay_str) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Verify AP - AP: {}, Data: 0x{:X}",
                    A,
                    data,
                ))?);
                self.drive_header(*A, true, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(data, &mut nodes, &parity_compare, verifies, captures)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            Attrs::SWDWriteDP(data, dp_addr, ack, overlays, overlay_str) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Write DP - DP: {}, Data: 0x{:X}",
                    dp_addr,
                    data,
                ))?);
                self.drive_header(*dp_addr, false, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.drive_data(data, &mut nodes, overlays, overlay_str)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            // Attrs::SWDBufferedWriteDP => {
            //     // ...
            // },
            Attrs::SWDVerifyDP(data, dp_addr, ack, parity_compare, verifies, captures, overlay, overlay_str) => {
                let mut nodes = vec!();
                nodes.push(comment(&format!(
                    "SWD: Verify DP - DP: {}, Data: 0x{:X}",
                    dp_addr,
                    data,
                ))?);
                self.drive_header(*dp_addr, false, true, &mut nodes)?;
                self.verify_ack(ack, &mut nodes)?;
                self.verify_data(data, &mut nodes, &parity_compare, verifies, captures)?;
                self.close_swd(&mut nodes)?;
                Ok(Return::Inline(nodes))
            },
            Attrs::SWDLineReset => {
                let mut nodes = vec!();
                nodes.push(comment("SWD: Line Reset")?);
                nodes.push(set_pin_drive_high(&self.swdclk)?);
                nodes.push(set_pin_drive_high(&self.swdio)?);
                nodes.push(repeat(50)?);
                nodes.push(set_pin_highz(&self.swdio)?);
                nodes.push(repeat(2)?);
                Ok(Return::Inline(nodes))
            },
            _ => Ok(Return::ProcessChildren),
        }
    }
}
