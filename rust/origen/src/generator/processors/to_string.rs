//! This is used to implement the fmt::Display trait for nodes and is a
//! good example of a simple AST processor

use crate::generator::ast::*;
use crate::generator::processor::*;

pub struct ToString {
    indent: usize,
    output: String,
}

impl ToString {
    pub fn run(node: &Node) -> String {
        let mut p = ToString {
            indent: 0,
            output: "".to_string(),
        };
        node.process(&mut p);
        p.output
    }
}

impl Processor for ToString {
    fn on_all(&mut self, node: &Node) -> Return {
        self.output += &" ".repeat(self.indent);
        self.output += &format!("{:?}\n", node.attrs);
        self.indent += 4;
        node.process_children(self);
        self.indent -= 4;
        Return::Unmodified
    }
}
