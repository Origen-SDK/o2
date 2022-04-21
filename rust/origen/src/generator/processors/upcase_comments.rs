//! A very simple example processor which returns a new version of the
//! given AST with all comments changed to upper case.

use super::super::nodes::PAT;
use crate::Result;
use origen_metal::ast::*;

pub struct UpcaseComments {}

impl UpcaseComments {
    #[allow(dead_code)]
    pub fn run(node: &Node<PAT>) -> Result<Node<PAT>> {
        let mut p = UpcaseComments {};
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<PAT> for UpcaseComments {
    fn on_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        match &node.attrs {
            PAT::Comment(level, msg) => {
                let new_node = node.replace_attrs(PAT::Comment(*level, msg.to_uppercase()));
                Ok(Return::Replace(new_node))
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::nodes::PAT;

    #[test]
    fn it_works() {
        let mut ast = AST::new();
        ast.push_and_open(node!(PAT::Test, "t1".to_string()));
        ast.push(node!(PAT::Cycle, 1, false));
        ast.push(node!(PAT::Comment, 1, "some comment".to_string()));

        let mut expect = AST::new();
        expect.push_and_open(node!(PAT::Test, "t1".to_string()));
        expect.push(node!(PAT::Cycle, 1, false));
        expect.push(node!(PAT::Comment, 1, "SOME COMMENT".to_string()));

        assert_eq!(
            UpcaseComments::run(&ast.to_node()).expect("Comments upcased"),
            expect
        );
    }
}
