use crate::generator::ast::*;
use crate::generator::processor::*;
use super::mem_ap::MemAP;
use crate::services::swd::Acknowledgements;
use crate::{Result, Error};

macro_rules! biguint32 {
    ( $num:expr ) => {{
        num_bigint::BigUint::from($num as u32)
    }};
}

/// Transforms ArmDebugMemAP read & write transactions into SWD/JTAG transactions
pub struct ArmDebugMemAPsToProtocol {
    processing_mem_ap: bool,
    current_mem_ap: Option<MemAP>,
    swdnjtag: bool,
}

impl ArmDebugMemAPsToProtocol {
    pub fn run(node: &Node) -> Result<Node> {
        Ok(node.process(&mut Self{
            processing_mem_ap: false,
            current_mem_ap: None,
            swdnjtag: true,
        })?.unwrap())
    }

    pub fn select_dp_reg(&self, reg: &str) -> Result<Vec<Node>> {
        let mut dp_trans = vec![];
        self.write_dp_reg(
            "SELECT",
            match reg {
                "CTRL/STAT" => 0,
                "DLCR" => 1,
                "TARGETID" => 2,
                "DLPIDR" => 3,
                "EVENTSTAT" => 4,
                _ => return Err(Error::new(&format!("Arm Debug: Unknown DP register '{}'", reg)))
            },
        )?;
        Ok(dp_trans)
    }

    pub fn write_dp_reg(&self, reg: &str, data: u32) -> Result<Vec<Node>> {
        let mut dp_trans = vec![];
        match reg {
            "DPIDR" => dp_trans.push(node!(SWDWriteAP, biguint32!(0), 0, Acknowledgements::Ok)),
            "SELECT" => dp_trans.push(node!(SWDWriteDP, biguint32!(0x8), data, Acknowledgements::Ok)),
            "RDBUFF" => return Err(Error::new(&format!("Arm Debug DP register 'RDBUFF' is RO (read-only)! Cannot write this register."))),
            _ => {
                dp_trans.append(&mut self.select_dp_reg(reg)?);
                dp_trans.push(node!(SWDWriteDP, biguint32!(0x4), data, Acknowledgements::Ok));
            }
        }
        Ok(dp_trans)
    }
}

impl Processor for ArmDebugMemAPsToProtocol {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::ArmDebugMemAPWriteReg(mem_ap) => {
                self.processing_mem_ap = true;
                self.current_mem_ap = Some(mem_ap.clone());
                Ok(Return::ProcessChildren)
            },
            Attrs::RegWrite(reg_id, data, overlay, overlay_name) => {
                if self.processing_mem_ap {
                    // Prototype only - coreyeng
                    let mut nodes: Vec<Node> = vec!();
                    let addr;
                    {
                        let dut = crate::dut();
                        let r = dut.get_register(*reg_id)?;
                        addr = num_bigint::BigUint::from(r.address(&dut, None)?);
                    }

                    // Use DP access to set the CSW
                    // Use DP access to set the TAR
                    //nodes.append(&mut self.write_dp_reg("SELECT", 0x4));

                    // 
                    nodes.push(crate::comment!("ArmDebug MemAP Register Write"));
                    nodes.push(node!(SWDWriteAP, addr, 0, Acknowledgements::Ok)); // Test write
                    nodes.push(node!(SWDWriteAP, data.clone(), 0, Acknowledgements::Ok)); // Test write
                    Ok(Return::Inline(nodes))
                } else {
                    Ok(Return::ProcessChildren)
                }
            },
            //Attrs::ArmDebugDPWrite()
            // Attrs::ArmDebugPowerUp(mem_ap) => {
            //     Ok(Return::Inline(self.write_dp_reg("CTRLSTAT", 0x5000_0000)?))
            // },
            _ => Ok(Return::ProcessChildren)
        }
    }

    fn on_end_of_block(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::ArmDebugMemAPWriteReg(mem_ap) => {
                self.processing_mem_ap = false;
                Ok(Return::None)
            },
            _ => Ok(Return::None)
        }
    }
}

/// Transforms ArmDebugSWJ nodes into pin state transactions
pub struct ArmDebugSwjToPinStates {}
