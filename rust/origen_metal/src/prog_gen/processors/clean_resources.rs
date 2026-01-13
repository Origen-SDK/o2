//! A simple example processor which removes all flow logic and redundant content from resources
//! blocks, basically leaving just the test definitions
use super::super::nodes::PGM;
use crate::ast::*;
use crate::Result;

pub struct CleanResources {
    in_resources: bool,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    let mut p = CleanResources { in_resources: false };
    Ok(node.process(&mut p)?.unwrap())
}

impl Processor<PGM> for CleanResources {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Resources => {
                let orig = self.in_resources;
                self.in_resources = true;
                let nodes = node.process_children(self)?;
                self.in_resources = orig;
                Ok(Return::ReplaceChildren(nodes))
            }
            PGM::Test(_, _) => {
                Ok(Return::ProcessChildren)
            }
            // For all other nodes
            _ => {
                if self.in_resources {
                    // Throw away everything except those with a handler defined above
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
        }
    }
}
