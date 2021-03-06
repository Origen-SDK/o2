//! A very simple example processor which returns a new version of the
//! given AST with all comments changed to upper case.

use crate::generator::ast::*;
use crate::generator::processor::*;

pub struct UpcaseComments {}

impl UpcaseComments {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = UpcaseComments {};
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for UpcaseComments {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::Comment(level, msg) => {
                let new_node = node.replace_attrs(Attrs::Comment(*level, msg.to_uppercase()));
                Ok(Return::Replace(new_node))
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::ast::*;
    use crate::generator::processors::*;

    #[test]
    fn it_works() {
        let mut ast = AST::new();
        ast.push_and_open(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Comment, 1, "some comment".to_string()));

        let mut expect = AST::new();
        expect.push_and_open(node!(Test, "t1".to_string()));
        expect.push(node!(Cycle, 1, false));
        expect.push(node!(Comment, 1, "SOME COMMENT".to_string()));

        assert_eq!(
            UpcaseComments::run(&ast.to_node()).expect("Comments upcased"),
            expect
        );
    }
}
