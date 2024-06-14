//! A simple example processor which will combine adjacent cycle nodes

use super::super::nodes::PAT;
use crate::Result;
use origen_metal::ast::{Node, Processor, Return};
use std::collections::HashMap;

pub struct CycleCombiner {
    cycle_count: u32,
}

impl CycleCombiner {
    pub fn run(node: &Node<PAT>) -> Result<Node<PAT>> {
        let mut p = CycleCombiner { cycle_count: 0 };
        Ok(node.process(&mut p)?.unwrap())
    }

    fn consume_cycles(&mut self) -> Node<PAT> {
        let cyc = node!(PAT::Cycle, self.cycle_count, true);
        self.cycle_count = 0;
        cyc
    }
}

impl Processor<PAT> for CycleCombiner {
    fn on_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        match &node.attrs {
            PAT::Cycle(repeat, compressable) => {
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
            _ => Ok(Return::ProcessChildren)
        }
    }
    
    fn on_processed_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        if self.cycle_count > 0 {
            let cyc = self.consume_cycles();
            if node.children.len() > 0 {
                let mut new_node = node.clone();
                new_node.add_child(cyc);
                Ok(Return::Replace(new_node))
            } else {
                Ok(Return::Inline(vec![cyc, node.clone()]))
            }
        } else {
            Ok(Return::Unmodified)
        }
    }
}

#[allow(non_snake_case)]
pub struct UnpackCaptures {
    // Captures
    pub captures__least_cycles_remaining: usize,
    pub capturing: HashMap<Option<usize>, (usize, Option<String>)>,

    // Overlays
    pub overlays__least_cycles_remaining: usize,
    pub overlaying: HashMap<Option<usize>, (usize, Option<String>, Option<String>)>,
}

impl UnpackCaptures {
    pub fn run(node: &Node<PAT>) -> Result<Node<PAT>> {
        let mut p = UnpackCaptures {
            captures__least_cycles_remaining: std::usize::MAX,
            capturing: HashMap::new(),
            overlays__least_cycles_remaining: std::usize::MAX,
            overlaying: HashMap::new(),
        };
        Ok(node.process(&mut p)?.unwrap())
    }

    fn is_capturing(&self) -> bool {
        self.capturing.len() > 0
    }

    fn is_overlaying(&self) -> bool {
        self.overlaying.len() > 0
    }
}

impl Processor<PAT> for UnpackCaptures {
    fn on_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        match &node.attrs {
            PAT::Capture(capture, _metadata) => {
                // Keep track of which pins we need to capture and for how long
                let cycles = capture.cycles.unwrap_or(1);
                if let Some(pids) = capture.pin_ids.as_ref() {
                    for pin in pids.iter() {
                        if self.capturing.contains_key(&Some(*pin)) {
                            // Already capturing this pin. Raise an error.
                            bail!(
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
                        if cycles < self.captures__least_cycles_remaining {
                            self.captures__least_cycles_remaining = cycles;
                        }
                    }
                } else {
                    if self.capturing.contains_key(&None) {
                        bail!(
                            "Generic capture is already occurring. Cannot initiate another capture"
                        );
                    }
                    self.capturing
                        .insert(None, (cycles, capture.symbol.clone()));
                    if cycles < self.captures__least_cycles_remaining {
                        self.captures__least_cycles_remaining = cycles;
                    }
                }
                Ok(Return::Unmodified)
            }
            PAT::Overlay(overlay, _metadata) => {
                // For unpacking an overlay, this is almost identical to a capture.
                let cycles = overlay.cycles.unwrap_or(1);
                if let Some(pids) = overlay.pin_ids.as_ref() {
                    for pin in pids.iter() {
                        if self.overlaying.contains_key(&Some(*pin)) {
                            // Already overlaying this pin. Raise an error.
                            bail!(
                                "Overlay requested on pin '{}' but this pin is already overlaying",
                                {
                                    let dut = crate::dut();
                                    let p = &dut.pins[*pin];
                                    p.name.clone()
                                }
                            );
                        }
                        self.overlaying.insert(
                            Some(*pin),
                            (cycles, overlay.label.clone(), overlay.symbol.clone()),
                        );
                        if cycles < self.overlays__least_cycles_remaining {
                            self.overlays__least_cycles_remaining = cycles;
                        }
                    }
                } else {
                    if self.overlaying.contains_key(&None) {
                        bail!(
                            "Generic overlay is already occurring. Cannot initiate another overlay"
                        );
                    }
                    self.overlaying.insert(
                        None,
                        (cycles, overlay.label.clone(), overlay.symbol.clone()),
                    );
                    if cycles < self.overlays__least_cycles_remaining {
                        self.overlays__least_cycles_remaining = cycles;
                    }
                }
                Ok(Return::Unmodified)
            }
            PAT::Cycle(repeat, compressable) => {
                if self.capturing.len() > 0 || self.overlaying.len() > 0 {
                    // De-compress the cycles to account for captures and overlays
                    let mut to_repeat = *repeat as usize;
                    let mut nodes: Vec<Node<PAT>> = vec![];
                    while to_repeat > 0 {
                        let mut this_cycle_captures: HashMap<Option<usize>, Option<String>> =
                            HashMap::new();
                        let mut this_cycle_overlays: HashMap<
                            Option<usize>,
                            (Option<String>, Option<String>),
                        > = HashMap::new();
                        let this_repeat;

                        let least_cycles_remaining;
                        if self.captures__least_cycles_remaining
                            < self.overlays__least_cycles_remaining
                        {
                            least_cycles_remaining = self.captures__least_cycles_remaining;
                        } else {
                            least_cycles_remaining = self.overlays__least_cycles_remaining;
                        }

                        if to_repeat >= least_cycles_remaining {
                            this_repeat = least_cycles_remaining;

                            if self.is_capturing() {
                                self.captures__least_cycles_remaining -= this_repeat;
                                if self.captures__least_cycles_remaining == 0 {
                                    self.captures__least_cycles_remaining = std::usize::MAX;
                                }
                            }

                            if self.is_overlaying() {
                                self.overlays__least_cycles_remaining -= this_repeat;
                                if self.overlays__least_cycles_remaining == 0 {
                                    self.overlays__least_cycles_remaining = std::usize::MAX;
                                }
                            }
                        } else {
                            this_repeat = to_repeat;
                            if self.is_capturing() {
                                self.captures__least_cycles_remaining -= this_repeat;
                            }
                            if self.is_overlaying() {
                                self.overlays__least_cycles_remaining -= this_repeat;
                            }
                        }
                        to_repeat -= this_repeat;

                        let mut finished_captures: Vec<Option<usize>> = vec![];
                        let mut finished_overlays: Vec<(Option<usize>, Option<String>)> = vec![];

                        // Decrease the cycle count for all captures
                        for (pin_id, cap) in self.capturing.iter_mut() {
                            if cap.0 <= this_repeat {
                                // This capture will be exhausted by the end of this node
                                // Remove it from the list to capture
                                finished_captures.push(*pin_id);
                            } else {
                                // This capture won't be exhausted, but will
                                // decrease the remaining cycles a bit.
                                cap.0 -= this_repeat;
                                if cap.0 > self.captures__least_cycles_remaining {
                                    self.captures__least_cycles_remaining = cap.0;
                                }
                            }
                            this_cycle_captures.insert(*pin_id, cap.1.clone());
                        }

                        // Do the same for overlays
                        for (pin_id, ovl) in self.overlaying.iter_mut() {
                            if ovl.0 <= this_repeat {
                                // This overlay will be exhausted by the end of this node
                                // Remove it from the list to overlay
                                finished_overlays.push((*pin_id, ovl.1.clone()));
                            } else {
                                // This overlay won't be exhausted, but will
                                // decrease the remaining cycles a bit.
                                ovl.0 -= this_repeat;
                                if ovl.0 > self.overlays__least_cycles_remaining {
                                    self.overlays__least_cycles_remaining = ovl.0;
                                }
                            }
                            this_cycle_overlays.insert(*pin_id, (ovl.1.clone(), ovl.2.clone()));
                        }

                        if this_cycle_overlays.len() > 0 {
                            for _ in 0..this_repeat {
                                nodes.push(node!(PAT::Cycle, 1 as u32, false));
                            }
                        } else {
                            nodes.push(node!(PAT::Cycle, this_repeat as u32, *compressable));
                        }
                        finished_captures.iter().for_each(|pin_id| {
                            self.capturing.remove(pin_id);
                            nodes.push(node!(PAT::EndCapture, pin_id.clone()));
                        });
                        finished_overlays.iter().for_each(|(pin_id, label)| {
                            self.overlaying.remove(pin_id);
                            nodes.push(node!(PAT::EndOverlay, label.clone(), pin_id.clone()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::nodes::PAT;
    use origen_metal::ast::AST;

    fn reg_write_node() -> Node<PAT> {
        let mut trans = crate::Transaction::new_write(0x12345678_u32.into(), 32).unwrap();
        trans.reg_id = Some(10);
        node!(PAT::RegWrite, trans)
    }

    #[test]
    fn it_works() {
        let mut ast = AST::new();
        ast.push(node!(PAT::Test, "cycle_combiner".to_string()));
        ast.push(node!(PAT::Comment, 1, "HELLO".to_string()));
        let id = ast.push_and_open(reg_write_node());
        ast.push(node!(PAT::Comment, 1, "SHOULD BE INSIDE REG TRANSACTION".to_string()));
        ast.push(node!(PAT::Cycle, 1, false));
        ast.push(node!(PAT::Cycle, 1, true));
        ast.push(node!(PAT::Cycle, 1, true));
        ast.push(node!(PAT::Cycle, 1, true));
        ast.push(node!(PAT::Cycle, 1, true));
        ast.push(node!(PAT::Cycle, 1, true));
        let _ = ast.close(id);
        ast.push(node!(PAT::Comment, 1, "SHOULD BE OUTSIDE REG TRANSACTION".to_string()));

        let combined = CycleCombiner::run(&ast.to_node()).expect("Cycles combined");

        let mut expect = AST::new();
        expect.push(node!(PAT::Test, "cycle_combiner".to_string()));
        expect.push(node!(PAT::Comment, 1, "HELLO".to_string()));
        let id = expect.push_and_open(reg_write_node());
        expect.push(node!(PAT::Comment, 1, "SHOULD BE INSIDE REG TRANSACTION".to_string()));
        expect.push(node!(PAT::Cycle, 1, false));
        expect.push(node!(PAT::Cycle, 5, true));
        let _ = expect.close(id);
        expect.push(node!(PAT::Comment, 1, "SHOULD BE OUTSIDE REG TRANSACTION".to_string()));

        assert_eq!(combined, expect.to_node());
    }

    #[test]
    fn it_leaves_something_behind() {
        let mut ast = AST::new();
        ast.push(node!(PAT::Test, "all_compressable".to_string()));
        ast.push(node!(PAT::SetTimeset, 0));
        ast.push(node!(PAT::SetPinHeader, 10));
        ast.push(node!(PAT::Cycle, 100, true));
        ast.push(node!(PAT::Comment, 1, "Producing Pattern".to_string()));
        ast.push(node!(PAT::Cycle, 10, true));
        ast.push(node!(PAT::Cycle, 10, true));
        ast.push(node!(PAT::PatternEnd));

        let combined = CycleCombiner::run(&ast.to_node()).expect("Cycles combined");

        let mut expect = AST::new();
        expect.push(node!(PAT::Test, "all_compressable".to_string()));
        expect.push(node!(PAT::SetTimeset, 0));
        expect.push(node!(PAT::SetPinHeader, 10));
        expect.push(node!(PAT::Cycle, 100, true));
        expect.push(node!(PAT::Comment, 1, "Producing Pattern".to_string()));
        expect.push(node!(PAT::Cycle, 20, true));
        expect.push(node!(PAT::PatternEnd));

        assert_eq!(combined, expect.to_node());
    }
}