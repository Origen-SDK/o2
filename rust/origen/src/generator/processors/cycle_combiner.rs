//! A simple example processor which will combine adjacent cycle nodes

use crate::generator::ast::*;
use crate::generator::processor::*;

pub struct CycleCombiner {
    cycle_count: u32,
}

impl CycleCombiner {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Node {
        let mut p = CycleCombiner { cycle_count: 0 };
        node.process(&mut p).unwrap()
    }
}

impl Processor for CycleCombiner {
    fn on_cycle(&mut self, repeat: u32, _node: &Node) -> Return {
        self.cycle_count += repeat;
        Return::None
    }

    // Don't let it leave an open block with cycles pending
    fn on_end_of_block(&mut self, _node: &Node) -> Return {
        if self.cycle_count > 0 {
            let cyc = Node::new(Attrs::Cycle(self.cycle_count));
            self.cycle_count = 0;
            Return::Replace(cyc)
        } else {
            Return::None
        }
    }

    // This will be called for all nodes except for cycles
    fn on_all(&mut self, node: &Node) -> Return {
        if self.cycle_count == 0 {
            Return::ProcessChildren
        } else {
            let cyc = Node::new(Attrs::Cycle(self.cycle_count));
            self.cycle_count = 0;
            let new_node = node.process_children(self);
            Return::InlineUnboxed(vec![cyc, new_node])
        }
    }
}
