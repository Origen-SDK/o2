use crate::prog_gen::{FlowCondition, FlowID, PGM};
use crate::Result;
use crate::ast::{Node, Processor, Return};
use std::collections::HashMap;

pub struct MissingIDs {
    ids: HashMap<FlowID, Node<PGM>>,
    referenced_ids: HashMap<FlowID, Vec<Node<PGM>>>,
    referenced_early: HashMap<FlowID, Vec<Node<PGM>>>,
}

pub fn run(node: &Node<PGM>) -> Result<()> {
    let mut p = MissingIDs {
        ids: HashMap::new(),
        referenced_ids: HashMap::new(),
        referenced_early: HashMap::new(),
    };
    let _ = node.process(&mut p)?;

    let mut failed = false;
    let mut msg = "".to_string();

    for (id, nodes) in &p.referenced_ids {
        if !p.ids.contains_key(id) {
            msg += &format!(
                "Test ID {} is referenced in the following lines, but it is never defined:",
                id
            );
            if crate::PROG_GEN_CONFIG.debug_enabled() {
                for node in nodes {
                    msg += &format!("\n  {}", node.meta_string());
                }
            } else {
                msg += "\n  run again with the --debug switch to see them";
            }
            failed = true;
            p.referenced_early.remove(id);
        }
    }
    for (id, nodes) in &p.referenced_early {
        msg += &format!("Test ID {} is referenced in the following lines:", id);
        if crate::PROG_GEN_CONFIG.debug_enabled() {
            for node in nodes {
                msg += &format!("\n  {}", node.meta_string());
            }
        } else {
            msg += "\n  run again with the --debug switch to see them";
        }
        msg += "\nbut it was not defined until later:";
        if crate::PROG_GEN_CONFIG.debug_enabled() {
            msg += &format!("\n  {}", p.ids[id].meta_string());
        } else {
            msg += "\n  run again with the --debug switch to see where";
        }
        failed = true;
    }
    if failed {
        bail!("{}", msg)
    } else {
        Ok(())
    }
}

impl Processor<PGM> for MissingIDs {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Test(_, id) | PGM::TestStr(_, id) | PGM::Cz(_, _, id) => {
                self.ids.insert(id.to_owned(), node.without_children());
                Return::ProcessChildren
            }
            PGM::Group(_, _, _, id) => {
                if let Some(id) = id {
                    self.ids.insert(id.to_owned(), node.without_children());
                }
                Return::ProcessChildren
            }
            PGM::Condition(cond) => {
                match cond {
                    FlowCondition::IfFailed(ids)
                    | FlowCondition::IfAnyFailed(ids)
                    | FlowCondition::IfAllFailed(ids)
                    | FlowCondition::IfPassed(ids)
                    | FlowCondition::IfAnyPassed(ids)
                    | FlowCondition::IfAllPassed(ids)
                    | FlowCondition::IfRan(ids)
                    | FlowCondition::UnlessRan(ids) => {
                        for id in ids {
                            if !id.is_external() {
                                if !self.referenced_ids.contains_key(id) {
                                    self.referenced_ids.insert(id.to_owned(), vec![]);
                                }
                                self.referenced_ids
                                    .get_mut(id)
                                    .unwrap()
                                    .push(node.without_children());
                                if !self.ids.contains_key(id) {
                                    if !self.referenced_early.contains_key(id) {
                                        self.referenced_early.insert(id.to_owned(), vec![]);
                                    }
                                    self.referenced_early
                                        .get_mut(id)
                                        .unwrap()
                                        .push(node.without_children());
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Return::ProcessChildren
            }
            _ => Return::ProcessChildren,
        })
    }
}
