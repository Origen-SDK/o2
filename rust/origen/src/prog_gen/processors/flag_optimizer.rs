use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::{FlowCondition, GroupType};
use std::collections::HashMap;

///  This processor eliminates the use of un-necessary flags between adjacent tests:
///  
/// Input:
///    PGMFlow("f1")
///        PGMTest(1, FlowID("t1"))
///        PGMOnFailed(FlowID("t1"))
///            PGMSetFlag("t1_FAILED", true, true)
///            PGMContinue
///        PGMCondition(IfFlag(["t1_FAILED"]))
///            PGMTest(2, FlowID("t2"))
///
/// Output:
///    PGMFlow("f1")
///        PGMTest(1, FlowID("t1"))
///        PGMOnFailed(FlowID("t1"))
///            PGMTest(2, FlowID("t2"))
///
pub struct FlagOptimizer {
    optimize_when_continue: bool,
    /// The number of times each flag is used
    nodes_to_inline: Vec<Box<Node>>,
}

pub fn run(node: &Node, optimize_when_continue: Option<bool>) -> Result<Node> {
    let optimize_when_continue = match optimize_when_continue {
        Some(x) => x,
        None => true,
    };
    let mut p = FlagOptimizer {
        optimize_when_continue: optimize_when_continue,
        nodes_to_inline: vec![],
    };
    let ast = node.process(&mut p)?.unwrap();

    let mut p = RemoveRedundantSetFlags {
        pass: 0,
        references: HashMap::new(),
    };
    let ast = ast.process(&mut p)?.unwrap();
    p.pass = 1;
    let ast = ast.process(&mut p)?.unwrap();

    Ok(ast)
}

/// Removes any auto-generated flags which are set in the flow but which are no longer used/referenced
#[derive(Debug)]
pub struct RemoveRedundantSetFlags {
    pass: usize,
    references: HashMap<String, usize>,
}

/// Extracts the IDs of all tests which have dependents on whether they passed, failed or ran
impl Processor for RemoveRedundantSetFlags {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        // Count all references the first time around
        if self.pass == 0 {
            Ok(match &node.attrs {
                Attrs::PGMCondition(cond) => match cond {
                    FlowCondition::IfFlag(ids) | FlowCondition::UnlessFlag(ids) => {
                        for id in ids {
                            if self.references.contains_key(id) {
                                let x = self.references[id];
                                self.references.insert(id.clone(), x + 1);
                            } else {
                                self.references.insert(id.clone(), 1);
                            }
                        }
                        Return::ProcessChildren
                    }
                    _ => Return::ProcessChildren,
                },
                _ => Return::ProcessChildren,
            })
        } else {
            Ok(match &node.attrs {
                Attrs::PGMSetFlag(flag, _, auto_generated) => {
                    if *auto_generated && !self.references.contains_key(flag) {
                        Return::None
                    } else {
                        Return::ProcessChildren
                    }
                }
                _ => Return::ProcessChildren,
            })
        }
    }
}

impl Processor for FlagOptimizer {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMGroup(_name, _, kind, _fid) => match kind {
                GroupType::Flow => {
                    let children = node.process_and_box_children(self)?;
                    Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
                }
                _ => Return::ProcessChildren,
            },
            Attrs::PGMFlow(_)
            | Attrs::PGMSubFlow(_)
            | Attrs::PGMElse
            | Attrs::PGMWhenever
            | Attrs::PGMWheneverAny
            | Attrs::PGMWheneverAll => {
                let children = node.process_and_box_children(self)?;
                Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFlag(_) | FlowCondition::UnlessFlag(_) => {
                    let children = node.process_and_box_children(self)?;
                    Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
                }
                _ => Return::ProcessChildren,
            },
            Attrs::PGMOnFailed(_) | Attrs::PGMOnPassed(_) => {
                let mut flag = None;
                let update = {
                    if let Some(to_inline) = self.nodes_to_inline.last() {
                        let mut i = 0;
                        if let Some(set_flag) = node.children.iter().find(|n| {
                            i += 1;
                            matches!(n.attrs, Attrs::PGMSetFlag(_, _, _))
                        }) {
                            if let Attrs::PGMSetFlag(flg, true, true) = &set_flag.attrs {
                                flag = Some(flg);
                                is_gated_by_set(flg, to_inline)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
                let mut children = node.process_and_box_children(self)?;
                if update {
                    let to_inline = self.nodes_to_inline.pop().unwrap();
                    let to_inline = self.reorder_nested_run_flags(flag.unwrap(), *to_inline)?;
                    for n in to_inline.children {
                        children.push(n);
                    }
                    self.nodes_to_inline.push(Box::new(node!(Nil))); // This will be popped off and discarded later
                }
                Return::Replace(node.updated(None, Some(self.optimize(children)?), None))
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl FlagOptimizer {
    fn optimize(&mut self, nodes: Vec<Box<Node>>) -> Result<Vec<Box<Node>>> {
        let mut results: Vec<Box<Node>> = vec![];
        let mut node1: Option<Box<Node>> = None;
        for node2 in nodes {
            let n2 = node2;
            if let Some(n1) = node1 {
                if self.can_be_combined(&n1, &n2) {
                    node1 = Some(self.combine(n1, n2)?);
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

    fn can_be_combined(&mut self, node1: &Box<Node>, node2: &Box<Node>) -> bool {
        // If node1 could have an OnFailed or OnPassed and if node2 is a flag condition
        if (matches!(node1.attrs, Attrs::PGMTest(_, _))
            || matches!(node1.attrs, Attrs::PGMSubFlow(_)))
            && (matches!(node2.attrs, Attrs::PGMCondition(FlowCondition::IfFlag(_)))
                || matches!(
                    node2.attrs,
                    Attrs::PGMCondition(FlowCondition::UnlessFlag(_))
                ))
        {
            // Don't optimize tests which are marked as continue if told not to
            let on_failed = node1
                .children
                .iter()
                .find(|n| matches!(n.attrs, Attrs::PGMOnFailed(_)));
            if let Some(on_failed) = on_failed {
                if !self.optimize_when_continue
                    && on_failed
                        .children
                        .iter()
                        .any(|n| matches!(n.attrs, Attrs::PGMContinue))
                {
                    return false;
                }
            }

            // Now return true if node 1 sets a flag that is gating node 2
            let on_pass_or_fail: Vec<&Box<Node>> = node1
                .children
                .iter()
                .filter(|n| {
                    matches!(n.attrs, Attrs::PGMOnFailed(_))
                        || matches!(n.attrs, Attrs::PGMOnPassed(_))
                })
                .collect();

            if on_pass_or_fail.iter().any(|n| {
                let set_flag = n
                    .children
                    .iter()
                    .find(|n| matches!(n.attrs, Attrs::PGMSetFlag(_, _, _)));
                if let Some(set_flag) = set_flag {
                    if let Attrs::PGMSetFlag(name, val, auto_generated) = &set_flag.attrs {
                        *val && is_gated_by_set(name, node2) && *auto_generated
                    } else {
                        unreachable!()
                    }
                } else {
                    false
                }
            }) {
                return true;
            }
        }
        false
    }

    fn combine(&mut self, node1: Box<Node>, node2: Box<Node>) -> Result<Box<Node>> {
        self.nodes_to_inline.push(node2);
        let node1 = node1.process_and_update_children(self)?;
        self.nodes_to_inline.pop();
        Ok(Box::new(node1))
    }

    // Returns the node with the run_flag clauses re-ordered to have the given flag of interest at the top.
    //
    // The caller guarantees the run_flag clause containing the given flag is present.
    //
    // For example, given this node:
    //
    //   s(:unless_flag, "flag1",
    //     s(:if_flag, "ot_BEA7F3B_FAILED",
    //       s(:test,
    //         s(:object, <TestSuite: inner_test1_BEA7F3B>),
    //         s(:name, "inner_test1_BEA7F3B"),
    //         s(:number, 0),
    //         s(:id, "it1_BEA7F3B"),
    //         s(:on_fail,
    //           s(:render, "multi_bin;")))))
    //
    // Then this node would be returned when the flag of interest is ot_BEA7F3B_FAILED:
    //
    //   s(:if_flag, "ot_BEA7F3B_FAILED",
    //     s(:unless_flag, "flag1",
    //       s(:test,
    //         s(:object, <TestSuite: inner_test1_BEA7F3B>),
    //         s(:name, "inner_test1_BEA7F3B"),
    //         s(:number, 0),
    //         s(:id, "it1_BEA7F3B"),
    //         s(:on_fail,
    //           s(:render, "multi_bin;")))))
    fn reorder_nested_run_flags(&mut self, flag: &str, node: Node) -> Result<Node> {
        // If the run_flag we care about is already at the top, just return node
        if let Attrs::PGMCondition(FlowCondition::IfFlag(flags)) = &node.attrs {
            if flags[0] == flag {
                return Ok(node);
            }
        }
        let mut p = RemoveIfFlag { flag: flag };
        let node = node.process(&mut p)?.unwrap();
        let n = node!(PGMCondition, FlowCondition::IfFlag(vec![flag.to_owned()]) => node);
        Ok(n)
    }
}

/// Removes any auto-generated flags which are set in the flow but which are no longer used/referenced
#[derive(Debug)]
pub struct RemoveIfFlag<'a> {
    flag: &'a str,
}

/// Extracts the IDs of all tests which have dependents on whether they passed, failed or ran
impl<'a> Processor for RemoveIfFlag<'a> {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFlag(ids) => {
                    if ids[0] == self.flag {
                        Return::UnwrapWithProcessedChildren
                    } else {
                        Return::ProcessChildren
                    }
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}

// node will always be an if_flag or unless_flag type node, guaranteed by the caller
//
// Returns true if flag matches the one supplied
//
//   s(:if_flag, flag,
//     s(:test, ...
//
// Also returns true if flag matches the one supplied, but it is nested within other flag conditions:
//
//   s(:unless_flag, other_flag,
//     s(:if_flag, other_flag2,
//       s(:if_flag, flag,
//         s(:test, ...
fn is_gated_by_set(flag: &str, node: &Box<Node>) -> bool {
    if let Attrs::PGMCondition(FlowCondition::IfFlag(flags)) = &node.attrs {
        if flags[0] == flag {
            return true;
        }
    }
    if node.children.len() == 1
        && (matches!(node.attrs, Attrs::PGMCondition(FlowCondition::IfFlag(_)))
            || matches!(
                node.attrs,
                Attrs::PGMCondition(FlowCondition::UnlessFlag(_))
            ))
    {
        return is_gated_by_set(flag, node.children.last().as_ref().unwrap());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::prog_gen::{BinType, FlowCondition, FlowID, GroupType};
    use crate::Result;

    #[test]
    fn basic_functionality_test() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2))
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMTest, 2, FlowID::from_int(2))
                ),
            ),
        );

        assert_eq!(output, run(&input, None)?);
        Ok(())
    }

    #[test]
    fn it_keeps_flags_with_later_references() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2))
            ),
            node!(PGMTest, 3, FlowID::from_int(3)),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 4, FlowID::from_int(4))
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                    node!(PGMTest, 2, FlowID::from_int(2))
                ),
            ),
            node!(PGMTest, 3, FlowID::from_int(3)),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 4, FlowID::from_int(4))
            ),
        );

        assert_eq!(output, run(&input, None)?);
        Ok(())
    }

    #[test]
    fn it_applies_the_optimization_within_nested_groups() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1) =>
                    node!(PGMOnFailed, FlowID::from_int(1) =>
                        node!(PGMContinue),
                        node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                    ),
                ),
                node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                    node!(PGMTest, 2, FlowID::from_int(2))
                ),
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1) =>
                    node!(PGMOnFailed, FlowID::from_int(1) =>
                        node!(PGMContinue),
                        node!(PGMTest, 2, FlowID::from_int(2)),
                    ),
                ),
            ),
        );

        assert_eq!(output, run(&input, None)?);
        Ok(())
    }

    #[test]
    fn a_more_complex_case_with_both_pass_and_fail_branches_to_be_optimized() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnPassed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_PASSED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2))
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 3, FlowID::from_int(3))
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMBin, 10, None, BinType::Bad)
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnPassed, FlowID::from_int(1) =>
                    node!(PGMTest, 2, FlowID::from_int(2))
                ),
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMTest, 3, FlowID::from_int(3)),
                    node!(PGMBin, 10, None, BinType::Bad),
                ),
            ),
        );

        assert_eq!(output, run(&input, None)?);
        Ok(())
    }

    #[test]
    fn nested_flags_are_handled() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                    node!(PGMTest, 2, FlowID::from_int(2))
                ),
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMCondition, FlowCondition::IfFlag(vec!["my_flag".to_string()]) =>
                        node!(PGMTest, 2, FlowID::from_int(2))
                    ),
                ),
            ),
        );

        assert_eq!(output, run(&input, None)?);
        Ok(())
    }

    #[test]
    fn optionally_doesnt_eliminate_flags_on_tests_with_a_continue() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2))
            ),
        );

        let output = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2))
            ),
        );

        assert_eq!(output, run(&input, Some(false))?);
        Ok(())
    }
}
