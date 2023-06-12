//! Adds referenced signal name to any value change nodes

use super::super::nodes::VCD;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;

#[allow(dead_code)]
pub struct Dereferencer {
    vars_reference: Vec<String>,
    vars_scope: Vec<Option<String>>
}

impl Dereferencer {
    #[allow(dead_code)]
    pub fn run(
        node: &Node<VCD>,
    ) -> Result<Node<VCD>> {
        let mut p = Dereferencer {
            vars_reference: Vec::new(),
            vars_scope: Vec::new()
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
            _ => Return::Unmodified
        };
        Ok(result)
    }
}