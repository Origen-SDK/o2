mod ast;
mod node;
mod processor;
mod processors;

pub use ast::AST;
pub use node::{Attrs, Meta, Node};
pub use processor::{Processor, Return};

pub fn to_string<T: Attrs>(node: &Node<T>) -> String {
    processors::ToString::run(node)
}
