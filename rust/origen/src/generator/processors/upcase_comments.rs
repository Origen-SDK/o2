//! A very simple example processor which returns a new version of the
//! given AST with all comments changed to upper case.

use crate::generator::ast::*;
use crate::generator::processor::*;

pub struct UpcaseComments {}

impl UpcaseComments {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Node {
        let mut p = UpcaseComments {};
        node.process(&mut p).unwrap()
    }
}

impl Processor for UpcaseComments {
    fn on_comment(&mut self, level: u8, msg: &str, node: &Node) -> Return {
        let new_node = node.replace_attrs(Attrs::Comment(level, msg.to_uppercase()));
        Return::Replace(new_node)
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::ast::*;
    use crate::generator::processors::*;

    #[test]
    fn it_works() {
        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Comment, 1, "some comment".to_string()));

        let mut expect = AST::new(node!(Test, "t1".to_string()));
        expect.push(node!(Cycle, 1, false));
        expect.push(node!(Comment, 1, "SOME COMMENT".to_string()));

        assert_eq!(UpcaseComments::run(&ast.to_node()), expect);
    }
}
