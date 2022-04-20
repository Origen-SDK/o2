//! A simple example processor which will screen out all sections of the AST that
//! are not intended for the given tester

use super::super::nodes::Pattern;
use crate::testers::SupportedTester;
use crate::Result;
use origen_metal::ast::*;

pub struct TargetTester {
    tester: SupportedTester,
}

impl TargetTester {
    #[allow(dead_code)]
    pub fn run(node: &Node<Pattern>, tester: SupportedTester) -> Result<Node<Pattern>> {
        let mut p = TargetTester { tester: tester };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<Pattern> for TargetTester {
    fn on_node(&mut self, node: &Node<Pattern>) -> Result<Return<Pattern>> {
        match &node.attrs {
            Pattern::TesterEq(testers) => {
                if testers
                    .iter()
                    .any(|t_eq| self.tester.is_compatible_with(t_eq))
                {
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::None)
                }
            }
            Pattern::TesterNeq(testers) => {
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
