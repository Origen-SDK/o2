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
        node.process_children(self)?;
        self.indent -= 4;
        Ok(Return::Unmodified)
    }
}
