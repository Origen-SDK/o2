use crate::prog_gen::{FlowCondition, GroupType, PGM};
use crate::Result;
use origen_metal::ast::{Node, Processor, Return};

/// This optimizes the condition nodes such that any adjacent flow nodes that
/// have the same condition, will be grouped together under a single condition
/// wrapper.
///
/// Input:
///
///    PGMFlow("f1")
///      PGMGroup("g1", None, GroupType::Flow, None)
///          PGMTest(1, FlowID("t1"))
///          PGMCondition(IfFlag(["bitmap"]))
///              PGMTest(2, FlowID("t2"))
///      PGMCondition(IfFlag(["bitmap"]))
///        PPPGMGroup("g1", None, GroupType::Flow, None)
///            PGMCondition(IfFlag(["x"]))
///                PGMTest(3, FlowID("t3"))
///            PGMCondition(IfFlag(["y"]))
///                PGMCondition(IfFlag(["x"]))
///                    PGMTest(4, FlowID("t4"))
///
/// Output:
///
///    PGMFlow("f1")
///        PGMGroup("g1", None, GroupType::Flow, None)
///            PGMTest(1, FlowID("t1"))
///            PGMCondition(IfFlag(["bitmap"]))
///                PGMTest(2, FlowID("t2"))
///                PGMCondition(IfFlag(["x"]))
///                    PGMTest(3, FlowID("t3"))
///                    PGMCondition(IfFlag(["y"]))
///                        PGMTest(4, FlowID("t4"))
///
pub struct Condition {
    volatiles: Vec<String>,
    conditions_to_remove: Vec<Node<PGM>>,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    let mut p = Condition {
        volatiles: vec![],
        conditions_to_remove: vec![],
    };
    let ast = node.process(&mut p)?.unwrap();

    Ok(ast)
}

impl Processor<PGM> for Condition {
    fn on_node(&mut self, node: &Node<PGM>) -> origen_metal::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Volatile(flag) => {
                self.volatiles.push(flag.to_owned());
                Return::Unmodified
            }
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                let children = node.process_and_box_children(self)?;
                Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
            }
            PGM::Group(_name, _, kind, _fid) => match kind {
                GroupType::Flow => {
                    if self
                        .conditions_to_remove
                        .iter()
                        .any(|n| n.attrs == node.attrs)
                    {
                        let children = node.process_and_box_children(self)?;
                        Return::InlineBoxed(self.optimize(children)?)
                    } else {
                        // Remove any nested occurences of this same group
                        self.conditions_to_remove
                            .push(node.updated(None, Some(vec![]), None));
                        let children = node.process_and_box_children(self)?;
                        self.conditions_to_remove.pop();
                        Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
                    }
                }
                _ => Return::ProcessChildren,
            },
            PGM::Condition(cond) => {
                let volatile = match cond {
                    FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                        self.volatiles.contains(&flags[0])
                    }
                    _ => false,
                };
                if volatile {
                    let children = node.process_and_box_children(self)?;
                    Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
                } else {
                    if self
                        .conditions_to_remove
                        .iter()
                        .any(|n| n.attrs == node.attrs)
                    {
                        let children = node.process_and_box_children(self)?;
                        Return::InlineBoxed(self.optimize(children)?)
                    } else {
                        // Remove any nested occurences of this same group
                        self.conditions_to_remove
                            .push(node.updated(None, Some(vec![]), None));
                        let children = node.process_and_box_children(self)?;
                        self.conditions_to_remove.pop();
                        Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
                    }
                }
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl Condition {
    fn optimize(&mut self, nodes: Vec<Box<Node<PGM>>>) -> Result<Vec<Box<Node<PGM>>>> {
        let mut results: Vec<Box<Node<PGM>>> = vec![];
        let mut node1: Option<Box<Node<PGM>>> = None;
        for node2 in nodes {
            let n2 = node2;
            if let Some(n1) = node1 {
                if self.can_be_combined(&n1, &n2) {
                    node1 = Some(Box::new(self.combine(n1, n2)?.process(self)?.unwrap()));
                } else {
                    results.push(n1);
                    node1 = Some(n2);
                }
            } else {
                node1 = Some(n2);
            }
        }
        if let Some(n) = node1 {
            results.push(n);
        }
        Ok(results)
    }

    fn can_be_combined(&mut self, node1: &Box<Node<PGM>>, node2: &Box<Node<PGM>>) -> bool {
        if is_condition_node(node1) && is_condition_node(node2) {
            let n2_conditions = self.conditions(node2);
            self.conditions(node1)
                .iter()
                .any(|n| n2_conditions.iter().any(|n2| n.attrs == n2.attrs))
        } else {
            false
        }
    }

    fn combine(&mut self, node1: Box<Node<PGM>>, node2: Box<Node<PGM>>) -> Result<Box<Node<PGM>>> {
        let mut n1_conditions = self.conditions(&node1);
        let n2_conditions = self.conditions(&node2);
        n1_conditions.retain(|n1| n2_conditions.iter().any(|n2| n1.attrs == n2.attrs));
        let n = n1_conditions.len();
        for node in &n1_conditions {
            self.conditions_to_remove.push(node.to_owned());
        }
        let node1 = node1.process(self)?.unwrap();
        let node2 = node2.process(self)?.unwrap();
        for _ in 0..n {
            self.conditions_to_remove.pop();
        }
        let mut node = n1_conditions.pop().unwrap();
        node.add_children(vec![node1, node2]);
        n1_conditions.reverse();
        for mut n in n1_conditions {
            n.add_child(node);
            node = n;
        }
        Ok(Box::new(node))
    }

    fn conditions(&self, node: &Box<Node<PGM>>) -> Vec<Node<PGM>> {
        let mut results = vec![];
        match &node.attrs {
            PGM::Condition(cond) => match cond {
                FlowCondition::IfEnable(flags)
                | FlowCondition::UnlessEnable(flags)
                | FlowCondition::IfFlag(flags)
                | FlowCondition::UnlessFlag(flags) => {
                    let flag = &flags[0];
                    if !self.volatiles.contains(flag) {
                        results.push(node.updated(None, Some(vec![]), None));
                        // If potentially another condition is a direct child
                        if node.children.len() == 1 {
                            let mut r = self.conditions(node.children.first().unwrap());
                            results.append(&mut r);
                        }
                    }
                }
                _ => {
                    results.push(node.updated(None, Some(vec![]), None));
                    // If potentially another condition is a direct child
                    if node.children.len() == 1 {
                        let mut r = self.conditions(node.children.first().unwrap());
                        results.append(&mut r);
                    }
                }
            },
            PGM::Group(_, _, kind, _) => {
                if matches!(kind, GroupType::Flow) {
                    results.push(node.updated(None, Some(vec![]), None));
                    if node.children.len() == 1 {
                        let mut r = self.conditions(node.children.first().unwrap());
                        results.append(&mut r);
                    }
                }
            }
            _ => {}
        }
        results
    }
}

fn is_condition_node(node: &Box<Node<PGM>>) -> bool {
    matches!(node.attrs, PGM::Condition(_))
        || matches!(node.attrs, PGM::Group(_, _, GroupType::Flow, _))
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::prog_gen::{FlowCondition, FlowID, GroupType, PGM};
    use crate::Result;

    #[test]
    fn wraps_adjacent_nodes_that_share_the_same_conditions() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2))
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3))
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn wraps_nested_conditions() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["bitmap".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["bitmap".to_string()]) =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                        node!(PGM::Test, 4, FlowID::from_int(4)),
                    ),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["bitmap".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                        node!(PGM::Test, 4, FlowID::from_int(4)),
                    ),
                ),

            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn optimizes_groups_too() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Group, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
                node!(PGM::Group, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGM::Group, "G3".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g3")) =>
                        node!(PGM::Test, 4, FlowID::from_int(4)),
                    ),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::Group, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                    node!(PGM::Group, "G3".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g3")) =>
                        node!(PGM::Test, 4, FlowID::from_int(4)),
                    ),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn combined_condition_and_group_test() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                        node!(PGM::Test, 3, FlowID::from_int(3)),
                    ),
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                        node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                            node!(PGM::Test, 4, FlowID::from_int(4)),
                        ),
                    ),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                        node!(PGM::Test, 3, FlowID::from_int(3)),
                        node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                            node!(PGM::Test, 4, FlowID::from_int(4)),
                        ),
                    ),

                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn optimizes_jobs() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string()]) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                        node!(PGM::Test, 3, FlowID::from_int(3)),
                    ),
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                        node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                            node!(PGM::Test, 4, FlowID::from_int(4)),
                        ),
                    ),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["bitmap".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["x".to_string()]) =>
                        node!(PGM::Test, 3, FlowID::from_int(3)),
                        node!(PGM::Condition, FlowCondition::IfFlag(vec!["y".to_string()]) =>
                            node!(PGM::Test, 4, FlowID::from_int(4)),
                        ),
                    ),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn job_optimization_test_2() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Test, 4, FlowID::from_int(4)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
            node!(PGM::Test, 6, FlowID::from_int(6)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 7, FlowID::from_int(7)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Test, 4, FlowID::from_int(4)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
            node!(PGM::Test, 6, FlowID::from_int(6)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 7, FlowID::from_int(7)),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn job_optimization_test_3() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["p1".to_string(), "p2".to_string()]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn test_result_optimization_test() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
            node!(PGM::Log, "Embedded conditional tests 1".to_string()),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(5)]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(5)]) =>
                node!(PGM::Test, 7, FlowID::from_int(7)),
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(7)]) =>
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(5)]) =>
                    node!(PGM::Test, 8, FlowID::from_int(8)),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
            node!(PGM::Log, "Embedded conditional tests 1".to_string()),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(5)]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
                node!(PGM::Test, 7, FlowID::from_int(7)),
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(7)]) =>
                    node!(PGM::Test, 8, FlowID::from_int(8)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn test_result_optimization_test_2() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Log, "Test that if_any_failed works".to_string()),
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(1), FlowID::from_int(2)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Log, "Test the block form of if_any_failed".to_string()),
            node!(PGM::Test, 4, FlowID::from_int(4)),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(4), FlowID::from_int(5)]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
            ),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(4), FlowID::from_int(5)]) =>
                node!(PGM::Test, 7, FlowID::from_int(7)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Log, "Test that if_any_failed works".to_string()),
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(1), FlowID::from_int(2)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Log, "Test the block form of if_any_failed".to_string()),
            node!(PGM::Test, 4, FlowID::from_int(4)),
            node!(PGM::Test, 5, FlowID::from_int(5)),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(4), FlowID::from_int(5)]) =>
                node!(PGM::Test, 6, FlowID::from_int(6)),
                node!(PGM::Test, 7, FlowID::from_int(7)),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn adjacent_group_optimization_test() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "additional_erase".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["additional_erase".to_string()]) =>
                    node!(PGM::Condition, FlowCondition::IfJob(vec!["fr".to_string()]) =>
                        node!(PGM::Test, 1, FlowID::from_int(1)),
                    ),
                ),
            ),
            node!(PGM::Group, "additional_erase".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Condition, FlowCondition::IfJob(vec!["fr".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "additional_erase".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Condition, FlowCondition::IfJob(vec!["fr".to_string()]) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["additional_erase".to_string()]) =>
                            node!(PGM::Test, 1, FlowID::from_int(1)),
                    ),
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn removes_duplicate_conditions() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["data_collection".to_string()]) =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["data_collection".to_string()]) =>
                    node!(PGM::Test, 1, FlowID::from_int(1)),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["data_collection".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn flag_conditions_are_not_optimized_when_marked_as_volatile() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1) =>
                    node!(PGM::OnFailed, FlowID::from_int(1) =>
                        node!(PGM::SetFlag, "$My_Mixed_Flag".to_string(), true, false),
                        node!(PGM::Continue),
                    ),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["$My_Mixed_Flag".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1) =>
                    node!(PGM::OnFailed, FlowID::from_int(1) =>
                        node!(PGM::SetFlag, "$My_Mixed_Flag".to_string(), true, false),
                        node!(PGM::Continue),
                    ),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["$My_Mixed_Flag".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        assert_eq!(output, run(&input)?);

        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "my_flag".to_string()),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1) =>
                    node!(PGM::OnFailed, FlowID::from_int(1) =>
                        node!(PGM::SetFlag, "$My_Mixed_Flag".to_string(), true, false),
                        node!(PGM::Continue),
                    ),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["$My_Mixed_Flag".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "my_flag".to_string()),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1) =>
                    node!(PGM::OnFailed, FlowID::from_int(1) =>
                        node!(PGM::SetFlag, "$My_Mixed_Flag".to_string(), true, false),
                        node!(PGM::Continue),
                    ),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["$My_Mixed_Flag".to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);

        Ok(())
    }
}
