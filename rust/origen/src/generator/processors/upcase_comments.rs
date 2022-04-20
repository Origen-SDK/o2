//! A very simple example processor which returns a new version of the
//! given AST with all comments changed to upper case.

use super::super::nodes::Pattern;
use crate::Result;
use origen_metal::ast::*;

pub struct UpcaseComments {}

impl UpcaseComments {
    #[allow(dead_code)]
    pub fn run(node: &Node<Pattern>) -> Result<Node<Pattern>> {
        let mut p = UpcaseComments {};
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<Pattern> for UpcaseComments {
    fn on_node(&mut self, node: &Node<Pattern>) -> Result<Return<Pattern>> {
        match &node.attrs {
            Pattern::Comment(level, msg) => {
                let new_node = node.replace_attrs(Pattern::Comment(*level, msg.to_uppercase()));
                Ok(Return::Replace(new_node))
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::nodes::Pattern;
    use origen_metal::ast::*;

    #[test]
    fn it_works() {
        let mut ast = AST::new();
        ast.push_and_open(node!(Pattern::Test, "t1".to_string()));
        ast.push(node!(Pattern::Cycle, 1, false));
        ast.push(node!(Pattern::Comment, 1, "some comment".to_string()));

        let mut expect = AST::new();
        expect.push_and_open(node!(Pattern::Test, "t1".to_string()));
        expect.push(node!(Pattern::Cycle, 1, false));
        expect.push(node!(Pattern::Comment, 1, "SOME COMMENT".to_string()));

        assert_eq!(
            UpcaseComments::run(&ast.to_node()).expect("Comments upcased"),
            expect
        );
    }
}
