use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowCondition;
use crate::prog_gen::FlowID;
use std::collections::HashMap;

/// This processor will apply the relationships between tests, e.g. if testB should only
/// execute if testA passes, then this processor will update the AST to make testA set
/// a flag on pass, and then update testB to only run if that flag is set.
pub struct Relationship {
    test_results: HashMap<FlowID, Vec<TestResult>>,
}

pub fn run(node: &Node) -> Result<Node> {
    let mut p = ExtractTestResults {
        results: HashMap::new(),
    };
    let _ = node.process(&mut p)?;

    let mut p = Relationship {
        test_results: p.results,
    };
    let ast = node.process(&mut p)?.unwrap();

    Ok(ast)
}

enum TestResult {
    Passed,
    Failed,
    Ran,
}

pub struct ExtractTestResults {
    results: HashMap<FlowID, Vec<TestResult>>,
}

/// Extracts the IDs of all tests which have dependents on whether they passed, failed or ran
impl Processor for ExtractTestResults {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFailed(ids)
                | FlowCondition::IfAnyFailed(ids)
                | FlowCondition::IfAllFailed(ids)
                | FlowCondition::IfAnySitesFailed(ids)
                | FlowCondition::IfAllSitesFailed(ids) => {
                    for id in ids {
                        if !self.results.contains_key(id) {
                            self.results.insert(id.to_owned(), vec![]);
                        }
                        self.results.get_mut(id).unwrap().push(TestResult::Failed);
                    }
                    Return::ProcessChildren
                }
                FlowCondition::IfPassed(ids)
                | FlowCondition::IfAnyPassed(ids)
                | FlowCondition::IfAllPassed(ids)
                | FlowCondition::IfAnySitesPassed(ids)
                | FlowCondition::IfAllSitesPassed(ids) => {
                    for id in ids {
                        if !self.results.contains_key(id) {
                            self.results.insert(id.to_owned(), vec![]);
                        }
                        self.results.get_mut(id).unwrap().push(TestResult::Passed);
                    }
                    Return::ProcessChildren
                }
                FlowCondition::IfRan(ids) | FlowCondition::UnlessRan(ids) => {
                    for id in ids {
                        if !self.results.contains_key(id) {
                            self.results.insert(id.to_owned(), vec![]);
                        }
                        self.results.get_mut(id).unwrap().push(TestResult::Ran);
                    }
                    Return::ProcessChildren
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}

fn process_test_results(fid: &FlowID, mut node: Node, processor: &Relationship) -> Result<Return> {
    // If this test/group/sub-flow has a dependent
    if processor.test_results.contains_key(fid) {
        for r in &processor.test_results[fid] {
            match r {
                TestResult::Passed => {
                    node.ensure_node_present(Attrs::PGMOnPassed(fid.clone()));
                    node.ensure_node_present(Attrs::PGMOnFailed(fid.clone()));
                    for n in &mut node.children {
                        match n.attrs {
                            Attrs::PGMOnPassed(_) => {
                                n.children.push(Box::new(node!(
                                    PGMSetFlag,
                                    format!("{}_PASSED", fid.to_str()),
                                    true,
                                    true
                                )));
                            }
                            Attrs::PGMOnFailed(_) => {
                                let contains_delayed = n
                                    .children
                                    .iter()
                                    .any(|n| matches!(n.attrs, Attrs::PGMDelayed));
                                if !contains_delayed {
                                    n.ensure_node_present(Attrs::PGMContinue);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                TestResult::Failed => {
                    node.ensure_node_present(Attrs::PGMOnFailed(fid.clone()));
                    for n in &mut node.children {
                        match n.attrs {
                            Attrs::PGMOnFailed(_) => {
                                n.children.push(Box::new(node!(
                                    PGMSetFlag,
                                    format!("{}_FAILED", fid.to_str()),
                                    true,
                                    true
                                )));
                                let contains_delayed = n
                                    .children
                                    .iter()
                                    .any(|n| matches!(n.attrs, Attrs::PGMDelayed));
                                if !contains_delayed {
                                    n.ensure_node_present(Attrs::PGMContinue);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                TestResult::Ran => {
                    let set_flag = node!(PGMSetFlag, format!("{}_RAN", fid.to_str()), true, true);
                    match &node.attrs {
                        // For a test, set a flag immediately after the referenced test has executed
                        // but don't change its pass/fail handling
                        Attrs::PGMTest(_, _) | Attrs::PGMTestStr(_, _) => {
                            return Ok(Return::Inline(vec![node, set_flag]));
                        }
                        // For a group, set a flag immediately upon entry to the group to signal that
                        // it ran to later tests, this is better than doing it immediately after the group
                        // in case it was bypassed
                        Attrs::PGMGroup(_, _, _, _) | Attrs::PGMSubFlow(_, _) => {
                            node.insert_child(set_flag, node.children.len())?;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
    Ok(Return::Replace(node))
}

fn ids_to_flags(ids: &Vec<FlowID>, name: &str) -> Vec<String> {
    ids.iter()
        .map(|id| format!("{}_{}", id.to_str(), name))
        .collect()
}

impl Processor for Relationship {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMTest(_, fid) | Attrs::PGMTestStr(_, fid) => {
                let node = node.process_and_update_children(self)?;
                process_test_results(fid, node, &self)?
            }
            Attrs::PGMGroup(_, _, _, fid) | Attrs::PGMSubFlow(_, fid) => {
                if let Some(fid) = fid {
                    let node = node.process_and_update_children(self)?;
                    process_test_results(fid, node, &self)?
                } else {
                    Return::ProcessChildren
                }
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFailed(ids) | FlowCondition::IfAnyFailed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "FAILED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAnySitesFailed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfAnySitesFlag(
                            ids_to_flags(ids, "FAILED"),
                        ))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllSitesFailed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfAllSitesFlag(
                            ids_to_flags(ids, "FAILED"),
                        ))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllFailed(ids) => {
                    let mut n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(vec![format!(
                            "{}_FAILED",
                            ids.last().unwrap()
                        )]))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    for (i, id) in ids.into_iter().rev().enumerate() {
                        // The first id is already done above
                        if i != 0 {
                            n = node!(PGMCondition, FlowCondition::IfFlag(vec![format!("{}_FAILED", id)]) => n);
                        }
                    }
                    Return::Replace(n)
                }
                FlowCondition::IfPassed(ids) | FlowCondition::IfAnyPassed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "PASSED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAnySitesPassed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfAnySitesFlag(
                            ids_to_flags(ids, "PASSED"),
                        ))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllSitesPassed(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfAllSitesFlag(
                            ids_to_flags(ids, "PASSED"),
                        ))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllPassed(ids) => {
                    let mut n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(vec![format!(
                            "{}_PASSED",
                            ids.last().unwrap()
                        )]))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    for (i, id) in ids.into_iter().rev().enumerate() {
                        // The first id is already done above
                        if i != 0 {
                            n = node!(PGMCondition, FlowCondition::IfFlag(vec![format!("{}_PASSED", id)]) => n);
                        }
                    }
                    Return::Replace(n)
                }
                FlowCondition::IfRan(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "RAN",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::UnlessRan(ids) => {
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::UnlessFlag(
                            ids_to_flags(ids, "RAN"),
                        ))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::prog_gen::{BinType, FlowCondition, FlowID, GroupType};
    use crate::Result;

    #[test]
    fn it_updates_both_sides_of_the_relationship() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad)
                ),
            ),
            node!(PGMCondition, FlowCondition::IfPassed(vec![FlowID::from_int(1)]) =>
                node!(PGMTest, 3, FlowID::from_int(3))
            ),
            node!(PGMCondition, FlowCondition::IfPassed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 4, FlowID::from_int(4))
            ),
            node!(PGMCondition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 4, FlowID::from_int(5))
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnPassed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                ),
            ),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad),
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGMOnPassed, FlowID::from_int(2) =>
                    node!(PGMSetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_PASSED".to_string()]) =>
                node!(PGMTest, 3, FlowID::from_int(3))
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t2_PASSED".to_string()]) =>
                node!(PGMTest, 4, FlowID::from_int(4))
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t2_FAILED".to_string()]) =>
                node!(PGMTest, 4, FlowID::from_int(5))
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn embedded_test_results_are_processed() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMCondition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                node!(PGMTest, 2, FlowID::from_int(2)),
                node!(PGMTest, 3, FlowID::from_int(3)),
                node!(PGMCondition, FlowCondition::IfFailed(vec![FlowID::from_int(3)]) =>
                    node!(PGMTest, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                    node!(PGMContinue),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGMTest, 2, FlowID::from_int(2)),
                node!(PGMTest, 3, FlowID::from_int(3) =>
                    node!(PGMOnFailed, FlowID::from_int(3) =>
                        node!(PGMSetFlag, "t3_FAILED".to_string(), true, true),
                        node!(PGMContinue),
                    ),
                ),
                node!(PGMCondition, FlowCondition::IfFlag(vec!["t3_FAILED".to_string()]) =>
                    node!(PGMTest, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn any_failed_is_processed() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2)),
            node!(PGMCondition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(1), FlowID::from_int(2)]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_FAILED".to_string(), true, true),
                    node!(PGMContinue),
                ),
            ),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMSetFlag, "t2_FAILED".to_string(), true, true),
                    node!(PGMContinue),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string(), "t2_FAILED".to_string()]) =>
                node!(PGMTest, 3, FlowID::from_int(3))
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn group_based_if_failed_is_processed() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1)),
                node!(PGMTest, 2, FlowID::from_int(2)),
            ),
            node!(PGMCondition, FlowCondition::IfFailed(vec![FlowID::from_str("g1")]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
                node!(PGMTest, 4, FlowID::from_int(4)),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1)),
                node!(PGMTest, 2, FlowID::from_int(2)),
                node!(PGMOnFailed, FlowID::from_str("g1") =>
                    node!(PGMSetFlag, "g1_FAILED".to_string(), true, true),
                    node!(PGMContinue),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["g1_FAILED".to_string()]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
                node!(PGMTest, 4, FlowID::from_int(4)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn group_based_if_passed_is_processed() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1)),
                node!(PGMTest, 2, FlowID::from_int(2)),
            ),
            node!(PGMCondition, FlowCondition::IfPassed(vec![FlowID::from_str("g1")]) =>
                node!(PGMGroup, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGMTest, 3, FlowID::from_int(3)),
                    node!(PGMTest, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMGroup, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGMTest, 1, FlowID::from_int(1)),
                node!(PGMTest, 2, FlowID::from_int(2)),
                node!(PGMOnPassed, FlowID::from_str("g1") =>
                    node!(PGMSetFlag, "g1_PASSED".to_string(), true, true),
                ),
                node!(PGMOnFailed, FlowID::from_str("g1") =>
                    node!(PGMContinue),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["g1_PASSED".to_string()]) =>
                node!(PGMGroup, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGMTest, 3, FlowID::from_int(3)),
                    node!(PGMTest, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn ran_conditions_are_converted_to_flag_conditions() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2)),
            node!(PGMCondition, FlowCondition::UnlessRan(vec![FlowID::from_int(1)]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfRan(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 4, FlowID::from_int(4)),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMSetFlag, "t1_RAN".to_string(), true, true),
            node!(PGMTest, 2, FlowID::from_int(2)),
            node!(PGMSetFlag, "t2_RAN".to_string(), true, true),
            node!(PGMCondition, FlowCondition::UnlessFlag(vec!["t1_RAN".to_string()]) =>
                    node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t2_RAN".to_string()]) =>
                    node!(PGMTest, 4, FlowID::from_int(4)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_add_continue_to_the_parent_test_if_it_is_already_set_to_delayed() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad),
                    node!(PGMDelayed),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfPassed(vec![FlowID::from_int(1)]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfPassed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 4, FlowID::from_int(4)),
            ),
            node!(PGMCondition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 5, FlowID::from_int(5)),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnPassed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                ),
            ),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad),
                    node!(PGMDelayed),
                    node!(PGMSetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGMOnPassed, FlowID::from_int(2) =>
                    node!(PGMSetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t1_PASSED".to_string()]) =>
                    node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t2_PASSED".to_string()]) =>
                    node!(PGMTest, 4, FlowID::from_int(4)),
            ),
            node!(PGMCondition, FlowCondition::IfFlag(vec!["t2_FAILED".to_string()]) =>
                    node!(PGMTest, 5, FlowID::from_int(5)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn if_any_site_conditions_work() -> Result<()> {
        let input = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1)),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfAnySitesPassed(vec![FlowID::from_int(1)]) =>
                node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfAllSitesPassed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 4, FlowID::from_int(4)),
            ),
            node!(PGMCondition, FlowCondition::IfAnySitesFailed(vec![FlowID::from_int(2)]) =>
                node!(PGMTest, 5, FlowID::from_int(5)),
            ),
        );

        let expected = node!(PGMFlow, "f1".to_string() =>
            node!(PGMTest, 1, FlowID::from_int(1) =>
                node!(PGMOnPassed, FlowID::from_int(1) =>
                    node!(PGMSetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGMOnFailed, FlowID::from_int(1) =>
                    node!(PGMContinue),
                ),
            ),
            node!(PGMTest, 2, FlowID::from_int(2) =>
                node!(PGMOnFailed, FlowID::from_int(2) =>
                    node!(PGMBin, 10, None, BinType::Bad),
                    node!(PGMContinue),
                    node!(PGMSetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGMOnPassed, FlowID::from_int(2) =>
                    node!(PGMSetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGMCondition, FlowCondition::IfAnySitesFlag(vec!["t1_PASSED".to_string()]) =>
                    node!(PGMTest, 3, FlowID::from_int(3)),
            ),
            node!(PGMCondition, FlowCondition::IfAllSitesFlag(vec!["t2_PASSED".to_string()]) =>
                    node!(PGMTest, 4, FlowID::from_int(4)),
            ),
            node!(PGMCondition, FlowCondition::IfAnySitesFlag(vec!["t2_FAILED".to_string()]) =>
                    node!(PGMTest, 5, FlowID::from_int(5)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }
}
