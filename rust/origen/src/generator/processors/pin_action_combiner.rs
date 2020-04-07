//! A simple processor which will combine pin actions not separated by a cycle into a single PinAction node

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::core::model::pins::pin::PinActions;
use crate::core::model::pins::StateTracker;
use std::collections::HashMap;
use crate::Result;

/// Combines adjacent pin actions into a single pin action
pub struct PinActionCombiner {
    current_state: HashMap<String, (PinActions, u8)>,
    i: usize,
    first_pass: bool,
    updated_indices: Vec<usize>,
}

/// Combines pin actions so that only pin actions which change the pin states are left.
/// Note: this processor assumes that anything that touches PinAction nodes has already completed.
/// Because some form of lookahead is needed, and to avoid missing actions that may occur in chlid nodes,
///  this processor is run in two passes:
///   First, all nodes are run through and indices of pin changes are marked.
///   Second, all non-PinAction nodes are copied over, with only PinAction nodes whose indices were marked are copied over.
impl PinActionCombiner {
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = PinActionCombiner {
          current_state: HashMap::new(),
          i: 0,
          first_pass: true,
          updated_indices: vec![],
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
            Attrs::PinAction(pin_changes) => {
              if self.first_pass {
                // Compare to the last seen pin actions. If these are the same, then this node can be deleted on the next pass.
                // If they're different, then these updates must be saved.
                let save_node = false;
                for (n, state) in pin_changes.iter() {
                  if let Some(_state) = self.current_state.get_mut(n) {
                    if _state.0 != state.0 || _state.1 != state.1 {
                      // Update both in case they both changed. Quicker just to do the update than to add a bunch of conditionals.
                      _state.0 = state.0;
                      _state.1 = state.1;
                      self.updated_indices.push(self.i);
                    }
                  } else {
                    self.current_state.insert(n.to_string(), *state);
                    self.updated_indices.push(self.i);
                  }
                }
                self.i += 1;
                Ok(Return::Unmodified)
              } else {
                // Delete any pin action nodes whose indice is not presetn in the saved indices
                if self.updated_indices.contains(&self.i) {
                  self.i += 1;
                  Ok(Return::Unmodified)
                } else {
                  self.i += 1;
                  Ok(Return::None)
                }
              }
            },
            _ => {
                // For all other nodes, just advance.
                self.i += 1;
                Ok(Return::ProcessChildren)
            }
        }
    }
}
