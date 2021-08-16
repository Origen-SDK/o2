use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowID;
use std::collections::HashMap;

pub struct DuplicateIDs {
    ids: HashMap<FlowID, Node>,
}

pub fn run(node: &Node) -> Result<()> {
    let mut p = DuplicateIDs {
        ids: HashMap::new(),
    };
    let _ = node.process(&mut p)?;
    Ok(())
}

impl DuplicateIDs {
    fn validate_id(&mut self, id: &FlowID, node: &Node) -> Result<()> {
        if self.ids.contains_key(id) {
            if crate::STATUS.is_debug_enabled() {
                return error!(
                    "The ID '{}' was assigned more than once, by the following flow lines:\n{}\n{}",
                    id,
                    self.ids[id].meta_string(),
                    node.meta_string()
                );
            } else {
                return error!(
                    "The ID '{}' was assigned more than once, run again with the --debug \
                 switch enabled to trace back to a flow line",
                    id
                );
            }
        } else {
            self.ids
                .insert(id.to_owned(), node.replace_children(vec![]));
        }
        Ok(())
    }
}

impl Processor for DuplicateIDs {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMTest(_, id) | Attrs::PGMTestStr(_, id) | Attrs::PGMCz(_, _, id) => {
                self.validate_id(id, node)?;
                Return::ProcessChildren
            }
            Attrs::PGMGroup(_, _, _, id) => {
                if let Some(id) = id {
                    self.validate_id(id, node)?;
                }
                Return::ProcessChildren
            }
            _ => Return::ProcessChildren,
        })
    }
}
