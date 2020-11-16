//! A simple example processor which will screen out all sections of the AST that
//! are not intended for the given tester

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::testers::SupportedTester;

pub struct TargetTester {
    tester: SupportedTester,
}

impl TargetTester {
    #[allow(dead_code)]
    pub fn run(node: &Node, tester: SupportedTester) -> Result<Node> {
        let mut p = TargetTester { tester: tester };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for TargetTester {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::TesterEq(testers) => {
                if testers
                    .iter()
                    .any(|t_eq| self.tester.is_compatible_with(t_eq))
                {
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::None)
                }
            }
            Attrs::TesterNeq(testers) => {
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
