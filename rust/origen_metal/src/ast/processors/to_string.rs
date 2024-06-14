//! This is used to implement the fmt::Display trait for nodes and is a
//! good example of a simple AST processor

use crate::ast::{node::Attrs, Node, Processor, Return};
use crate::Result;

pub struct ToString<T> {
    indent: usize,
    output: String,
    // Had to use T somewhere in here to get it to compile, gave up trying to find
    // a more elegant solution
    _not_used: Option<T>,
}

impl<T: Attrs> ToString<T> {
    pub fn run(node: &Node<T>) -> String {
        let mut p = ToString {
            indent: 0,
            output: "".to_string(),
            _not_used: None,
        };
        node.process(&mut p).unwrap();
        p.output
    }
}

impl<T: Attrs> Processor<T> for ToString<T> {
    fn on_node(&mut self, node: &Node<T>) -> Result<Return<T>> {
        self.output += &" ".repeat(self.indent);
        self.output += &format!("{}\n", node.attrs);
        self.indent += 4;
        Ok(Return::ProcessChildren)
    }

    fn on_processed_node(&mut self, _node: &Node<T>) -> Result<Return<T>> {
        self.indent -= 4;
        Ok(Return::None)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AST;

    #[derive(Clone, PartialEq, Serialize, Debug)]
    pub enum STRTEST {
        Root,
        Integer(i64),
        Float(f64),
        String(String),
    }

    impl std::fmt::Display for STRTEST {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                _ => write!(f, "{}", format!("{:?}", self)),
            }
        }
    }

    #[test]
    fn test_process_children() {
        let mut ast: AST<STRTEST> = AST::new();
        let mut ids: Vec<usize> = vec![];
        ids.push(ast.push_and_open(node!(STRTEST::Root)));
        ast.push(node!(STRTEST::String, "Hello World!".to_string()));
        ast.push(node!(STRTEST::Integer, 2001));
        ids.push(ast.push_and_open(node!(STRTEST::Float, 97.1)));
        ast.push(node!(STRTEST::String, "Test Indent".to_string()));
        let _ = ast.close(ids.pop().unwrap());
        ast.push(node!(STRTEST::String, "Pop back".to_string()));
        let _ = ast.close(ids.pop().unwrap());

        let expect = "Root
    String(\"Hello World!\")
    Integer(2001)
    Float(97.1)
        String(\"Test Indent\")
    String(\"Pop back\")\n".to_string();

        assert_eq!(ast.to_string(), expect);
    }
}