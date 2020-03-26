//! A simple example processor which will combine adjacent cycle nodes

use crate::generator::ast::*;
use crate::generator::processor::*;

pub struct CycleCombiner {
    cycle_count: u32,
}

impl CycleCombiner {
    #[allow(dead_code)]
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = CycleCombiner { cycle_count: 0 };
        Ok(node.process(&mut p)?.unwrap())
    }

    fn consume_cycles(&mut self) -> Node {
        let cyc = node!(Cycle, self.cycle_count, true);
        self.cycle_count = 0;
        cyc
    }
}

impl Processor for CycleCombiner {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::Cycle(repeat, compressable) => {
                if *compressable {
                    self.cycle_count += repeat;
                    Ok(Return::None)
                } else {
                    if self.cycle_count > 0 {
                        let cyc = self.consume_cycles();
                        Ok(Return::Inline(vec![cyc, node.clone()]))
                    } else {
                        Ok(Return::Unmodified)
                    }
                }
            }
            // For all other nodes except for cycles
            _ => {
                if self.cycle_count == 0 {
                    Ok(Return::ProcessChildren)
                } else {
                    let cyc = self.consume_cycles();
                    let new_node = node.process_children(self)?;
                    Ok(Return::Inline(vec![cyc, new_node]))
                }
            }
        }
    }

    // Don't let it leave an open block with cycles pending
    fn on_end_of_block(&mut self, _node: &Node) -> Result<Return> {
        if self.cycle_count > 0 {
            Ok(Return::Replace(self.consume_cycles()))
        } else {
            Ok(Return::None)
        }
    }
}
