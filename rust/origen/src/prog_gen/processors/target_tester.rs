//! A simple example processor which will screen out all sections of the AST that
//! are not intended for the given tester

use super::super::nodes::PGM;
use crate::testers::SupportedTester;
use origen_metal::ast::*;
use origen_metal::Result;

pub struct TargetTester {
    tester: SupportedTester,
}

pub fn run(node: &Node<PGM>, tester: SupportedTester) -> Result<Node<PGM>> {
    let mut p = TargetTester { tester: tester };
    Ok(node.process(&mut p)?.unwrap())
}

impl Processor<PGM> for TargetTester {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::TesterEq(testers) => {
                if testers
                    .iter()
                    .any(|t_eq| self.tester.is_compatible_with(t_eq))
                {
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::None)
                }
            }
            PGM::TesterNeq(testers) => {
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
