//! A simple example processor which will screen out all sections of the AST that
//! are not intended for the given tester

use super::super::nodes::PAT;
use crate::testers::SupportedTester;
use origen_metal::ast::*;
use origen_metal::Result;

pub struct TargetTester {
    tester: SupportedTester,
}

impl TargetTester {
    #[allow(dead_code)]
    pub fn run(node: &Node<PAT>, tester: SupportedTester) -> Result<Node<PAT>> {
        let mut p = TargetTester { tester: tester };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<PAT> for TargetTester {
    fn on_node(&mut self, node: &Node<PAT>) -> Result<Return<PAT>> {
        match &node.attrs {
            PAT::TesterEq(testers) => {
                if testers
                    .iter()
                    .any(|t_eq| self.tester.is_compatible_with(t_eq))
                {
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::None)
                }
            }
            PAT::TesterNeq(testers) => {
                if testers
                    .iter()
                    .any(|t_neq| self.tester.is_compatible_with(t_neq))
                {
                    Ok(Return::None)
                } else {
                    Ok(Return::UnwrapWithProcessedChildren)
                }
            }
            // For all other nodes
            _ => Ok(Return::ProcessChildren),
        }
    }
}
