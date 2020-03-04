use super::super::*;

pub struct ToString {
    indent: usize,
    output: String,
}

impl ToString {
    pub fn new() -> ToString {
        ToString {
            indent: 0,
            output: "".to_string(),
        }
    }

    pub fn run(&mut self, node: &Node) -> &str {
        node.process(self);
        &self.output
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
