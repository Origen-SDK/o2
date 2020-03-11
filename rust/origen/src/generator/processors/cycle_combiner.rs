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

    fn consume_cycles(&mut self) -> Node {
        let cyc = node!(Cycle, self.cycle_count, true);
        self.cycle_count = 0;
        cyc
    }
}

impl Processor for CycleCombiner {
    fn on_cycle(&mut self, repeat: u32, compressable: bool, node: &Node) -> Return {
        if compressable {
            self.cycle_count += repeat;
            Return::None
        } else {
            if self.cycle_count > 0 {
                let cyc = self.consume_cycles();
                Return::Inline(vec![cyc, node.clone()])
            } else {
                Return::Unmodified
            }
        }
    }

    // Don't let it leave an open block with cycles pending
    fn on_end_of_block(&mut self, _node: &Node) -> Return {
        if self.cycle_count > 0 {
            Return::Replace(self.consume_cycles())
        } else {
            Return::None
        }
    }

    // This will be called for all nodes except for cycles
    fn on_all(&mut self, node: &Node) -> Return {
        if self.cycle_count == 0 {
            Return::ProcessChildren
        } else {
            let cyc = self.consume_cycles();
            let new_node = node.process_children(self);
            Return::Inline(vec![cyc, new_node])
        }
    }
}
