use crate::prog_gen::FlowID;
use crate::prog_gen::PGM;
use crate::Result;
use crate::ast::{Node, Processor, Return};
use std::collections::HashMap;

pub struct DuplicateIDs {
    ids: HashMap<FlowID, Node<PGM>>,
}

pub fn run(node: &Node<PGM>) -> Result<()> {
    let mut p = DuplicateIDs {
        ids: HashMap::new(),
    };
    let _ = node.process(&mut p)?;
    Ok(())
}

impl DuplicateIDs {
    fn validate_id(&mut self, id: &FlowID, node: &Node<PGM>) -> Result<()> {
        if self.ids.contains_key(id) {
            if crate::PROG_GEN_CONFIG.debug_enabled() {
                bail!(
                    "The ID '{}' was assigned more than once, by the following flow lines:\n{}\n{}",
                    id,
                    self.ids[id].meta_string(),
                    node.meta_string()
                );
            } else {
                bail!(
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

impl Processor<PGM> for DuplicateIDs {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Test(_, id) | PGM::TestStr(_, id) | PGM::Cz(_, _, id) => {
                self.validate_id(id, node)?;
                Return::ProcessChildren
            }
            PGM::Group(_, _, _, id) => {
                if let Some(id) = id {
                    self.validate_id(id, node)?;
                }
                Return::ProcessChildren
            }
            _ => Return::ProcessChildren,
        })
    }
}
