//! A simple example processor which will combine adjacent cycle nodes

use crate::generator::ast::*;
use crate::generator::processor::*;
use std::collections::HashMap;

pub struct CycleCombiner {
    cycle_count: u32,
}

impl CycleCombiner {
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
                    let new_node = node.process_and_update_children(self)?;
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

pub struct UnpackCaptures {
    pub least_cycles_remaining: usize,
    pub capturing: HashMap<Option<usize>, (usize, Option<String>)>,
}

impl UnpackCaptures {
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = UnpackCaptures {
            least_cycles_remaining: std::usize::MAX,
            capturing: HashMap::new(),
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for UnpackCaptures {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::Capture(capture, _metadata) => {
                // Keep track of which pins we need to capture and for how long
                let cycles = capture.cycles.unwrap_or(1);
                if let Some(pids) = capture.pin_ids.as_ref() {
                    for pin in pids.iter() {
                        if self.capturing.contains_key(&Some(*pin)) {
                            // Already capturing this pin. Raise an error.
                            return error!(
                                "Capture requested on pin '{}' but this pin is already capturing",
                                {
                                    let dut = crate::dut();
                                    let p = &dut.pins[*pin];
                                    p.name.clone()
                                }
                            );
                        }
                        self.capturing
                            .insert(Some(*pin), (cycles, capture.symbol.clone()));
                        if cycles < self.least_cycles_remaining {
                            self.least_cycles_remaining = cycles;
                        }
                    }
                } else {
                    if self.capturing.contains_key(&None) {
                        return error!(
                            "Generic capture is already occurring. Cannot initiate another capture"
                        );
                    }
                    self.capturing
                        .insert(None, (cycles, capture.symbol.clone()));
                }
                Ok(Return::Unmodified)
            }
            Attrs::Cycle(repeat, compressable) => {
                if self.capturing.len() > 0 {
                    // De-compress the cycles to account for captures
                    let mut to_repeat = *repeat as usize;
                    let mut nodes: Vec<Node> = vec![];
                    while to_repeat > 0 {
                        let mut this_cycle_captures: HashMap<Option<usize>, Option<String>> =
                            HashMap::new();
                        let this_repeat;
                        if to_repeat >= self.least_cycles_remaining {
                            this_repeat = self.least_cycles_remaining;
                            self.least_cycles_remaining = std::usize::MAX;
                        } else {
                            this_repeat = to_repeat;
                            self.least_cycles_remaining -= this_repeat;
                            if self.least_cycles_remaining == 0 {
                                self.least_cycles_remaining = std::usize::MAX;
                            }
                        }
                        to_repeat -= this_repeat;

                        let mut finished_captures: Vec<Option<usize>> = vec![];
                        for (pin_id, mut cap) in self.capturing.iter_mut() {
                            if cap.0 <= this_repeat {
                                // This capture will be exhausted by the end of this node
                                // Remove it from the list to capture
                                finished_captures.push(*pin_id);
                            } else {
                                // This capture won't be exhausted, but will
                                // decrease the remaining cycles a bit.
                                cap.0 -= this_repeat;
                                if cap.0 > self.least_cycles_remaining {
                                    self.least_cycles_remaining = cap.0;
                                }
                            }
                            this_cycle_captures.insert(*pin_id, cap.1.clone());
                        }
                        nodes.push(node!(Cycle, this_repeat as u32, *compressable));
                        finished_captures.iter().for_each(|pin_id| {
                            self.capturing.remove(pin_id);
                            nodes.push(node!(EndCapture, pin_id.clone()));
                        });
                    }
                    Ok(Return::Inline(nodes))
                } else {
                    // Not capturing anything. No need to update.
                    Ok(Return::Unmodified)
                }
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}
