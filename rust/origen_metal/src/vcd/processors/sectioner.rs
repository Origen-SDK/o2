//! Moves any comments between 'enddefinitions' node (in header) and start of 
//! data section to the beginning of the data section

use super::super::nodes::VCD;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;

pub struct Sectioner {
    collect_comments: bool,
    collected_comments: Vec<Node<VCD>>,
}

impl Sectioner {
    pub fn run(
        node: &Node<VCD>,
    ) -> Result<Node<VCD>> {
        let mut p = Sectioner {
            collect_comments: false,
            collected_comments: Vec::new()
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<VCD> for Sectioner {
    fn on_node(&mut self, node: &Node<VCD>) -> Result<Return<VCD>> {
        let result = match &node.attrs {
            VCD::Root => Return::ProcessChildren,
            VCD::HeaderSection => Return::ProcessChildren,
            VCD::EndDefinitions => {
                self.collect_comments = true;
                Return::Unmodified
            }
            VCD::Comment(val) => {
                if self.collect_comments{
                    self.collected_comments.push(node!(VCD::Comment, val.clone()));
                    Return::None
                }
                else {
                    Return::Unmodified
                }
            }
            VCD::DataSection => {
                self.collect_comments = false;

                // Insert all collected comments into data section
                let mut nodes = self.collected_comments.clone();
                let mut old_nodes = node.process_children(self)?;
                nodes.append(&mut old_nodes);

                Return::ReplaceChildren(nodes)
            }
            _ => Return::Unmodified
        };
        Ok(result)
    }
}