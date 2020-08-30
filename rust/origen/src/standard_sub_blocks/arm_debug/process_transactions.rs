use crate::generator::ast::*;
use crate::generator::processor::*;
use super::mem_ap::MemAP;
use crate::{Result, Error, swd_ok};
use num::ToPrimitive;
use crate::testers::vector_based::api::{read_data, drive_data, set_drive_high, set_pin_drive_low, set_highz, repeat, comment};

/// Transforms ArmDebugMemAP read & write transactions into SWD/JTAG transactions
pub struct ArmDebugMemAPsToProtocol {
    processing_mem_ap: bool,
    processing_internal_reg: bool,
    current_mem_ap: Option<MemAP>,
    jtagnswd: bool,
}

impl ArmDebugMemAPsToProtocol {
    pub fn run(node: &Node) -> Result<Node> {
        Ok(node.process(&mut Self{
            processing_mem_ap: false,
            processing_internal_reg: false,
            current_mem_ap: None,
            jtagnswd: true,
        })?.unwrap())
    }
}

impl Processor for ArmDebugMemAPsToProtocol {
    #[allow(non_snake_case)]
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::ArmDebugMemAPWriteReg(mem_ap) => {
                self.processing_mem_ap = true;
                self.current_mem_ap = Some(mem_ap.clone());
                Ok(Return::ProcessChildren)
            },
            Attrs::ArmDebugMemAPWriteInternalReg(_mem_ap) => {
                self.processing_mem_ap = true;
                self.processing_internal_reg = true;
                Ok(Return::ProcessChildren)
            },
            Attrs::RegWrite(reg_id, data, overlay, overlay_name) => {
                if self.processing_mem_ap {
                    let mut nodes: Vec<Node> = vec!();
                    if self.processing_internal_reg {
                        let addr;
                        {
                            let dut = crate::dut();
                            let r = dut.get_register(*reg_id)?;
                            addr = num_bigint::BigUint::from(r.address(&dut, None)?);
                        }
                        let select = addr.to_u32().unwrap() & 0xFFFF_FFF0;
                        let A = (addr.to_u32().unwrap() & 0xF) >> 2;

                        nodes.push(node!(
                            SWDWriteDP,
                            num_bigint::BigUint::from(select),
                            0x2,
                            swd_ok!(),
                            None,
                            None
                        ));
                    }
            //         // Prototype only - coreyeng
            //         let mut nodes: Vec<Node> = vec!();
            //         let addr;
            //         {
            //             let dut = crate::dut();
            //             let r = dut.get_register(*reg_id)?;
            //             addr = num_bigint::BigUint::from(r.address(&dut, None)?);
            //         }

            //         // Use DP access to set the CSW
            //         // Use DP access to set the TAR
            //         //nodes.append(&mut self.write_dp_reg("SELECT", 0x4));

            //         // 
            //         nodes.push(crate::comment!("ArmDebug MemAP Register Write"));
                    

            //         nodes.push(node!(SWDWriteAP, addr, 0, Acknowledgements::Ok)); // Test write
            //         nodes.push(node!(SWDWriteAP, data.clone(), 0, Acknowledgements::Ok)); // Test write
                    Ok(Return::Inline(nodes))
                } else {
                    Ok(Return::ProcessChildren)
                }
            },
            //Attrs::ArmDebugDPWrite()
            // Attrs::ArmDebugPowerUp(mem_ap) => {
            //     Ok(Return::Inline(self.write_dp_reg("CTRLSTAT", 0x5000_0000)?))
            // },
            Attrs::ArmDebugSwjJTAGToSWD(arm_debug) => {
                let swdio;
                let swdclk;
                {
                    let dut = crate::DUT.lock().unwrap();
                    swdio = dut._resolve_group_to_physical_pins(0, "swdio")?.first().unwrap().name.to_string();
                    swdclk = dut._resolve_group_to_physical_pins(0, "swdclk")?.first().unwrap().name.to_string();
                }
                let mut nodes: Vec<Node> = vec!();
                nodes.push(set_drive_high(vec!(&swdclk))?);
                nodes.push(set_drive_high(vec!(&swdio))?);
                nodes.push(repeat(50)?);
                nodes.append(&mut drive_data(
                    vec!(&swdio),
                    &num::BigUint::from(0xE79E as u32),
                    16,
                    true,
                    None::<&u8>,
                    None
                )?);
                nodes.push(repeat(55)?);
                nodes.push(comment("Move to IDLE")?);
                nodes.push(set_pin_drive_low(&swdio)?);
                nodes.push(repeat(4)?);
                Ok(Return::Inline(nodes))
            }
            _ => Ok(Return::ProcessChildren)
        }
    }

    fn on_end_of_block(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::ArmDebugMemAPWriteReg(_mem_ap) => {
                self.processing_mem_ap = false;
                Ok(Return::None)
            },
            _ => Ok(Return::None)
        }
    }
}

/// Transforms ArmDebugSWJ nodes into pin state transactions
pub struct ArmDebugSwjToPinStates {}
