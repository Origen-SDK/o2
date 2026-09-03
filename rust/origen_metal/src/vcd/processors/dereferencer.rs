//! Adds referenced signal name to any value change nodes.
//!
//! TODO: This processor is a stub. The intended behavior is to walk the header
//! section to build a map from identifier_code → (reference_name, scope), then
//! walk the data section and annotate each `ValueChange` node with the
//! corresponding signal name and scope. This would allow consumers to resolve
//! value changes back to named signals without having to maintain their own
//! lookup table.

use super::super::nodes::VCD;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;

#[allow(dead_code)]
pub struct Dereferencer {
    vars_reference: Vec<String>,
    vars_scope: Vec<Option<String>>,
}

impl Dereferencer {
    #[allow(dead_code)]
    pub fn run(node: &Node<VCD>) -> Result<Node<VCD>> {
        let mut p = Dereferencer {
            vars_reference: Vec::new(),
            vars_scope: Vec::new(),
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<VCD> for Dereferencer {
    fn on_node(&mut self, node: &Node<VCD>) -> Result<Return<VCD>> {
        let result = match &node.attrs {
            VCD::Root => Return::ProcessChildren,
            VCD::HeaderSection => Return::ProcessChildren,
            VCD::DataSection => Return::ProcessChildren,
            _ => Return::Unmodified,
        };
        Ok(result)
    }
}
