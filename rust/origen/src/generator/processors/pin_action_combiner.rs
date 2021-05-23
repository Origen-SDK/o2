//! A simple processor which will combine pin actions not separated by a cycle into a single PinAction node

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::Result;
use std::collections::HashMap;

/// Combines adjacent pin actions into a single pin action
pub struct PinActionCombiner {
    current_state: HashMap<usize, String>,
    i: usize,
    first_pass: bool,
    delete_current_index: bool,
    indices_to_delete: Vec<usize>,
}

/// Combines pin actions so that only pin actions which change the pin states are left.
/// Note: this processor assumes that anything that touches PinAction nodes has already completed.
/// Because some form of lookahead is needed, and to avoid missing actions that may occur in child nodes,
///  this processor is run in two passes:
///   First, all nodes are run through and indices of pin changes are marked.
///   Second, all non-PinAction nodes are copied over, with only PinAction nodes whose indices were marked are copied over.
impl PinActionCombiner {
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = PinActionCombiner {
            current_state: HashMap::new(),
            i: 0,
            first_pass: true,
            delete_current_index: false,
            indices_to_delete: vec![],
        };

        node.process(&mut p)?.unwrap();
        p.advance_to_second_pass();
        let n = node.process(&mut p)?.unwrap();
        Ok(n)
    }

    pub fn advance_to_second_pass(&mut self) {
        self.i = 0;
        self.first_pass = false;
    }
}

impl Processor for PinActionCombiner {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            // Grab the pins and push them into the running vectors, then delete this node.
            Attrs::PinGroupAction(_grp_id, _actions, _metadata) => {
                if self.first_pass {
                    // On the first pass, only mark the pin group's index
                    self.delete_current_index = true;
                    Ok(Return::ProcessChildren)
                } else {
                    // On the second pass, delete the pin group if necessary
                    if self.indices_to_delete.contains(&self.i) {
                        self.i += 1;
                        Ok(Return::None)
                    } else {
                        self.i += 1;
                        Ok(Return::Unmodified)
                    }
                }
            }
            Attrs::PinAction(pin_id, action, _metadata) => {
                if self.first_pass {
                    // Compare to the last seen pin actions. If these are the same, then this node can be deleted on the next pass.
                    // If they're different, then these updates must be saved.
                    if let Some(a) = self.current_state.get_mut(pin_id) {
                        if a != action {
                            self.delete_current_index = false;
                            self.current_state.insert(*pin_id, action.clone());
                        }
                    } else {
                        self.current_state.insert(*pin_id, action.clone());
                        self.delete_current_index = false;
                    }
                }
                Ok(Return::Unmodified)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }

    fn on_end_of_block(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::PinGroupAction(_grp_id, _actions, _metadata) => {
                if self.first_pass {
                    if self.delete_current_index {
                        self.indices_to_delete.push(self.i);
                    }
                    self.delete_current_index = false;
                }
                self.i += 1;
                Ok(Return::None)
            }
            _ => Ok(Return::None),
        }
    }
}
