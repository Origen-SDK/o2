//! A simple example processor which removes all flow logic and redundant content from resources
//! blocks, basically leaving just the test definitions.
//!
//! When keeping flows is enabled (for V93K SMT8), a ResourcesKeepFlows block is cleaned the same
//! way, except that:
//!
//! * the flow-type groups and sub-flows within it are kept whole, as flow definitions, the
//!   cleaning stops at their boundary. A BypassSubFlows node within the block is kept too, it
//!   marks the call of the flows within it as bypassed;
//! * the content which declares the flow's variables, the conditions on job, enables, flags and
//!   expressions, flag and variable assignments, variable declarations and loops, is kept in a
//!   FlowDataOnly node at the end of the block. It is analysed like executed content to work out
//!   the flow's variables, but it is not rendered.
//!
//! Every resources block comes out of this processor as a Resources node.
use super::super::nodes::PGM;
use crate::ast::*;
use crate::prog_gen::{FlowCondition, GroupType};
use crate::Result;

pub struct CleanResources {
    in_resources: bool,
    /// Keep the flows within the current resources block, when keeping flows is enabled
    keeping_flows: bool,
    /// Whether the flows within ResourcesKeepFlows blocks are kept, otherwise those blocks are
    /// cleaned like Resources blocks
    keep_flows_enabled: bool,
}

/// Cleans every resources block, keeping the flows within ResourcesKeepFlows blocks when
/// keep_flows is true, for a tester which writes flows to files of their own (V93K SMT8),
/// otherwise those blocks are cleaned like Resources blocks
pub fn run(node: &Node<PGM>, keep_flows: bool) -> Result<Node<PGM>> {
    let mut p = CleanResources {
        in_resources: false,
        keeping_flows: false,
        keep_flows_enabled: keep_flows,
    };
    Ok(node.process(&mut p)?.unwrap())
}

impl Processor<PGM> for CleanResources {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Resources | PGM::ResourcesKeepFlows => {
                let keeping =
                    self.keep_flows_enabled && matches!(node.attrs, PGM::ResourcesKeepFlows);
                let orig = self.in_resources;
                let orig_keeping = self.keeping_flows;
                self.in_resources = true;
                self.keeping_flows = keeping;
                let nodes = node.process_children(self);
                self.in_resources = orig;
                self.keeping_flows = orig_keeping;
                let mut nodes = nodes?;
                match node.attrs {
                    PGM::Resources => Ok(Return::ReplaceChildren(nodes)),
                    _ => {
                        if keeping {
                            let flow_data_only = node.process_children(&mut FlowDataOnly {})?;
                            if !flow_data_only.is_empty() {
                                nodes.push(Node::new_with_children(
                                    PGM::FlowDataOnly,
                                    flow_data_only,
                                ));
                            }
                        }
                        Ok(Return::Replace(
                            node.replace_attrs(PGM::Resources)
                                .replace_unboxed_children(nodes),
                        ))
                    }
                }
            }
            PGM::Test(_, _) => Ok(Return::ProcessChildren),
            PGM::Group(_, _, GroupType::Flow, _) | PGM::SubFlow(_, _)
                if self.in_resources && self.keeping_flows =>
            {
                // A flow definition, its content is processed as it is outside of resources
                self.in_resources = false;
                let nodes = node.process_children(self);
                self.in_resources = true;
                Ok(Return::ReplaceChildren(nodes?))
            }
            PGM::BypassSubFlows if self.in_resources && self.keeping_flows => {
                Ok(Return::ProcessChildren)
            }
            // For all other nodes
            _ => {
                if self.in_resources {
                    // Throw away everything except those with a handler defined above
                    Ok(Return::UnwrapWithProcessedChildren)
                } else {
                    Ok(Return::ProcessChildren)
                }
            }
        }
    }
}

/// Keeps the content of a ResourcesKeepFlows block which declares the flow's variables, see the
/// module documentation
struct FlowDataOnly {}

impl Processor<PGM> for FlowDataOnly {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        Ok(match &node.attrs {
            // The flows kept within the block declare their own variables, a resources block
            // within the block is not executed even when the block is
            PGM::Group(_, _, GroupType::Flow, _)
            | PGM::SubFlow(_, _)
            | PGM::Resources
            | PGM::ResourcesKeepFlows => Return::None,
            PGM::Condition(condition) => match condition {
                FlowCondition::IfJob(_)
                | FlowCondition::UnlessJob(_)
                | FlowCondition::IfEnable(_)
                | FlowCondition::UnlessEnable(_)
                | FlowCondition::IfFlag(_)
                | FlowCondition::UnlessFlag(_)
                | FlowCondition::IfExpr(_)
                | FlowCondition::UnlessExpr(_) => Return::ProcessChildren,
                // A condition on a test result would make a flag of the result of a test which
                // the block does not execute
                _ => Return::UnwrapWithProcessedChildren,
            },
            PGM::Loop(..)
            | PGM::Else
            | PGM::SetFlag(_, _, _)
            | PGM::SetVariable(_, _)
            | PGM::InputVariable(_)
            | PGM::OutputVariable(_)
            | PGM::LocalFlag(_) => Return::ProcessChildren,
            _ => Return::UnwrapWithProcessedChildren,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prog_gen::FlowID;

    fn block(keep_flows: bool) -> Node<PGM> {
        let mut block = if keep_flows {
            node!(PGM::ResourcesKeepFlows)
        } else {
            node!(PGM::Resources)
        };
        block.add_child(node!(PGM::Condition, FlowCondition::IfEnable(vec!["E".to_string()]) =>
            node!(PGM::Test, 1, FlowID::from_str("t1")),
            node!(PGM::Group, "shared".to_string(), None, GroupType::Flow, Some(FlowID::from_str("shared")) =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_str("t2"))
                )
            ),
            node!(PGM::SubFlow, "child".to_string(), None =>
                node!(PGM::Test, 3, FlowID::from_str("t3"))
            )
        ));
        node!(PGM::Flow, "main".to_string() => block)
    }

    #[test]
    fn the_flows_within_resources_which_keep_them_are_kept_when_enabled() -> Result<()> {
        let flattened = node!(PGM::Flow, "main".to_string() =>
            node!(PGM::Resources =>
                node!(PGM::Test, 1, FlowID::from_str("t1")),
                node!(PGM::Test, 2, FlowID::from_str("t2")),
                node!(PGM::Test, 3, FlowID::from_str("t3"))
            )
        );
        assert_eq!(run(&block(false), false)?, flattened);
        assert_eq!(run(&block(false), true)?, flattened);
        assert_eq!(run(&block(true), false)?, flattened);

        let kept = node!(PGM::Flow, "main".to_string() =>
            node!(PGM::Resources =>
                node!(PGM::Test, 1, FlowID::from_str("t1")),
                node!(PGM::Group, "shared".to_string(), None, GroupType::Flow, Some(FlowID::from_str("shared")) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                        node!(PGM::Test, 2, FlowID::from_str("t2"))
                    )
                ),
                node!(PGM::SubFlow, "child".to_string(), None =>
                    node!(PGM::Test, 3, FlowID::from_str("t3"))
                ),
                node!(PGM::FlowDataOnly =>
                    node!(PGM::Condition, FlowCondition::IfEnable(vec!["E".to_string()]))
                )
            )
        );
        assert_eq!(run(&block(true), true)?, kept);
        Ok(())
    }
}
