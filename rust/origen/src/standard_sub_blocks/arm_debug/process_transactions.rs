use crate::generator::ast::*;
use crate::generator::processor::*;
use super::mem_ap::MemAP;
use crate::services::swd::Acknowledgements;

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
                let mut nodes: Vec<Node> = vec!();
                let addr;
                {
                    let dut = crate::dut();
                    let r = dut.get_register(*reg_id)?;
                    addr = num_bigint::BigUint::from(r.address(&dut, None)?);
                }
                nodes.push(crate::comment!("ArmDebug MemAP Register Write"));
                nodes.push(node!(SWDWriteAP, addr, 0, Acknowledgements::Ok)); // Test write
                nodes.push(node!(SWDWriteAP, data.clone(), 0, Acknowledgements::Ok)); // Test write
                Ok(Return::Inline(nodes))
            }
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
