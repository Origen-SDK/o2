use crate::prog_gen::{FlowCondition, GroupType, PGM};
use crate::Result;
use crate::ast::{Node, Processor, Return};

/// This combines adjacent if flag nodes where the flag is in the opposite state
///
/// Input:
///
///    PGMFlow("f1")
///        PGMCondition(IfFlag(["SOME_FLAG"]))
///            PGMTest(1, FlowID("t1"))
///        PGMCondition(UnlessFlag(["SOME_FLAG"]))
///            PGMTest(2, FlowID("t2"))
///
/// Output:
///
///    PGMFlow("f1")
///        PGMCondition(IfFlag(["SOME_FLAG"]))
///            PGMTest(1, FlowID("t1"))
///            PGMElse
///                PGMTest(2, FlowID("t2"))
///
/// See here for an example of the kind of flow level effect it has:
/// https://github.com/Origen-SDK/origen_testers/issues/43
///
pub struct AdjacentIfCombiner {
    volatiles: Vec<String>,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    let mut p = AdjacentIfCombiner { volatiles: vec![] };
    let ast = node.process(&mut p)?.unwrap();

    Ok(ast)
}

impl Processor<PGM> for AdjacentIfCombiner {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Volatile(flag) => {
                self.volatiles.push(flag.to_owned());
                Return::Unmodified
            }
            PGM::Flow(_)
            | PGM::SubFlow(_, _)
            | PGM::Group(_, _, GroupType::Flow, _)
            | PGM::OnFailed(_)
            | PGM::OnPassed(_) => {
                let children = node.process_and_box_children(self)?;
                Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl AdjacentIfCombiner {
    fn optimize(&mut self, nodes: Vec<Box<Node<PGM>>>) -> Result<Vec<Box<Node<PGM>>>> {
        let mut results: Vec<Box<Node<PGM>>> = vec![];
        let mut node1: Option<Box<Node<PGM>>> = None;
        for node2 in nodes {
            let n2 = node2;
            if let Some(n1) = node1 {
                if self.is_opposite_flag_states(&n1, &n2) && self.is_safe_to_combine(&n1, &n2) {
                    results.push(self.combine(n1, n2)?);
                    node1 = None;
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

    fn combine(&mut self, node1: Box<Node<PGM>>, node2: Box<Node<PGM>>) -> Result<Box<Node<PGM>>> {
        let mut children = node1.process_and_box_children(self)?;
        let n2_children = node2.process_and_box_children(self)?;
        let else_node = node2.updated(Some(PGM::Else), Some(n2_children), None);
        children.push(Box::new(else_node));
        Ok(Box::new(node1.updated(None, Some(children), None)))
    }

    fn is_opposite_flag_states(&self, node1: &Box<Node<PGM>>, node2: &Box<Node<PGM>>) -> bool {
        if let PGM::Condition(FlowCondition::IfFlag(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::UnlessFlag(f2)) = &node2.attrs {
                return f1[0] == f2[0];
            }
        }
        if let PGM::Condition(FlowCondition::UnlessFlag(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::IfFlag(f2)) = &node2.attrs {
                return f1[0] == f2[0];
            }
        }
        if let PGM::Condition(FlowCondition::IfEnable(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::UnlessEnable(f2)) = &node2.attrs {
                return f1[0] == f2[0];
            }
        }
        if let PGM::Condition(FlowCondition::UnlessEnable(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::IfEnable(f2)) = &node2.attrs {
                return f1[0] == f2[0];
            }
        }
        false
    }

    /// Nodes won't be collapsed if node1 touches the shared run flag, i.e. if there is any chance
    /// that by the time it would naturally execute node2, the flag could have been changed by node1
    fn is_safe_to_combine(&self, node1: &Box<Node<PGM>>, _node2: &Box<Node<PGM>>) -> bool {
        match &node1.attrs {
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFlag(fl)
                | FlowCondition::UnlessFlag(fl)
                | FlowCondition::IfEnable(fl)
                | FlowCondition::UnlessEnable(fl) => {
                    let flag = &fl[0];
                    if !self.volatiles.contains(flag) || !ContainsATest::run(node1) {
                        return !ChangesFlag::run(node1, flag);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        false
    }
}

struct ChangesFlag<'a> {
    flag: &'a str,
    flag_found: bool,
}

impl<'a> ChangesFlag<'a> {
    fn run(node: &Node<PGM>, flag: &'a str) -> bool {
        let mut p = ChangesFlag {
            flag_found: false,
            flag: flag,
        };
        let _ = node.process(&mut p);
        p.flag_found
    }
}

impl<'a> Processor<PGM> for ChangesFlag<'a> {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::SetFlag(flag, _, _) | PGM::Enable(flag) | PGM::Disable(flag) => {
                if flag == self.flag {
                    self.flag_found = true;
                }
                Return::None
            }
            _ => {
                if self.flag_found {
                    Return::None
                } else {
                    Return::ProcessChildren
                }
            }
        })
    }
}

struct ContainsATest {
    test_found: bool,
}

impl ContainsATest {
    fn run(node: &Node<PGM>) -> bool {
        let mut p = ContainsATest { test_found: false };
        let _ = node.process(&mut p);
        p.test_found
    }
}

impl Processor<PGM> for ContainsATest {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Test(_, _) => {
                self.test_found = true;
                Return::None
            }
            _ => {
                if self.test_found {
                    Return::None
                } else {
                    Return::ProcessChildren
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::prog_gen::{BinType, FlowCondition, FlowID, PGM};
    use crate::Result;

    #[test]
    fn it_works() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Else =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_combine_if_there_is_potential_modification_of_the_flag_in_either_branch(
    ) -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::SetFlag, "SOME_FLAG".to_string(), true, false),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::SetFlag, "SOME_FLAG".to_string(), true, false),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["SOME_FLAG".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_combine_adjacent_nodes_based_on_a_volatile_flag_if_the_first_node_cannot_modify_the_flag(
    ) -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "my_flag".to_string()),
            // This section should combine, since does not contain any tests
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Bin, 1, None, BinType::Bad),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Bin, 2, None, BinType::Bad),
            ),
            node!(PGM::Test, 1, FlowID::from_int(1)),
            // This section should not combine, since does contain a tests which could potentially
            // change the state of the flag
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Bin, 2, None, BinType::Bad),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "my_flag".to_string()),
            // This section should combine, since does not contain any tests
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Bin, 1, None, BinType::Bad),
                node!(PGM::Else =>
                    node!(PGM::Bin, 2, None, BinType::Bad),
                ),
            ),
            node!(PGM::Test, 1, FlowID::from_int(1)),
            // This section should not combine, since does contain a tests which could potentially
            // change the state of the flag
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["my_flag".to_string()]) =>
                node!(PGM::Bin, 2, None, BinType::Bad),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }
}
