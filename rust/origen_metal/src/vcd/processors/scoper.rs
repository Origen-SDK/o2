//! Adds scope hierarchy to the AST as well as adds scope info to any var nodes

use super::super::nodes::VCD;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;

pub struct Scoper { 
    scope_path: Vec<String>,
}

impl Scoper {
    pub fn run(
        node: &Node<VCD>,
    ) -> Result<Node<VCD>> {
        let mut p = Scoper {
            scope_path: Vec::new()
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<VCD> for Scoper {
    fn on_node(&mut self, node: &Node<VCD>) -> Result<Return<VCD>> {
        let result = match &node.attrs {
            VCD::Root => Return::ProcessChildren,
            VCD::HeaderSection => {
                //let mut nodes = node.process_children(self)?;
                //Return::Replace(nodes.pop().unwrap())
                Return::ProcessChildren
            }
            VCD::Scope(_, val) => {
                self.scope_path.push(val.to_string());
                Return::Unmodified
            }
            VCD::UpScope => {
                self.scope_path.pop().expect("upscope should be matched in vcd header");
                Return::Unmodified
            }
            VCD::Var(t, size, identifier, reference, _) => {
                let var_scope = self.scope_path.join(".");
                let node = node!(VCD::Var, t.clone(), size.clone(), identifier.clone(), reference.clone(), Some(var_scope));
                Return::Replace(node)
            }
            VCD::DataSection => Return::ProcessChildren,
            _ => Return::Unmodified
        };
        Ok(result)
    }
}