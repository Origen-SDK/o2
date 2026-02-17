use crate::prog_gen::FlowID;
use crate::prog_gen::{FlowCondition, PGM};
use crate::Result;
use crate::ast::{Node, Processor, Return};
use std::collections::HashMap;

/// This processor will apply the relationships between tests, e.g. if testB should only
/// execute if testA passes, then this processor will update the AST to make testA set
/// a flag on pass, and then update testB to only run if that flag is set.
pub struct Relationship {
    test_results: HashMap<FlowID, Vec<TestResult>>,
}

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
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
impl Processor<PGM> for ExtractTestResults {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Condition(cond) => match cond {
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

fn process_test_results(
    fid: &FlowID,
    mut node: Node<PGM>,
    processor: &Relationship,
) -> Result<Return<PGM>> {
    // If this test/group/sub-flow has a dependent
    if processor.test_results.contains_key(fid) {
        for r in &processor.test_results[fid] {
            match r {
                TestResult::Passed => {
                    node.ensure_node_present(PGM::OnPassed(fid.clone()));
                    node.ensure_node_present(PGM::OnFailed(fid.clone()));
                    for n in &mut node.children {
                        match n.attrs {
                            PGM::OnPassed(_) => {
                                n.children.push(Box::new(node!(
                                    PGM::SetFlag,
                                    format!("{}_PASSED", fid.to_str()),
                                    true,
                                    true
                                )));
                            }
                            PGM::OnFailed(_) => {
                                let contains_delayed =
                                    n.children.iter().any(|n| matches!(n.attrs, PGM::Delayed));
                                if !contains_delayed {
                                    n.ensure_node_present(PGM::Continue);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                TestResult::Failed => {
                    node.ensure_node_present(PGM::OnFailed(fid.clone()));
                    for n in &mut node.children {
                        match n.attrs {
                            PGM::OnFailed(_) => {
                                n.children.push(Box::new(node!(
                                    PGM::SetFlag,
                                    format!("{}_FAILED", fid.to_str()),
                                    true,
                                    true
                                )));
                                let contains_delayed =
                                    n.children.iter().any(|n| matches!(n.attrs, PGM::Delayed));
                                if !contains_delayed {
                                    n.ensure_node_present(PGM::Continue);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                TestResult::Ran => {
                    let set_flag = node!(PGM::SetFlag, format!("{}_RAN", fid.to_str()), true, true);
                    match &node.attrs {
                        // For a test, set a flag immediately after the referenced test has executed
                        // but don't change its pass/fail handling
                        PGM::Test(_, _) | PGM::TestStr(_, _, _, _, _) => {
                            return Ok(Return::Inline(vec![node, set_flag]));
                        }
                        // For a group, set a flag immediately upon entry to the group to signal that
                        // it ran to later tests, this is better than doing it immediately after the group
                        // in case it was bypassed
                        PGM::Group(_, _, _, _) | PGM::SubFlow(_, _) => {
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

impl Processor<PGM> for Relationship {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Test(_, fid) | PGM::TestStr(_, fid, _, _, _) => {
                let node = node.process_and_update_children(self)?;
                process_test_results(fid, node, &self)?
            }
            PGM::Group(_, _, _, fid) | PGM::SubFlow(_, fid) => {
                if let Some(fid) = fid {
                    let node = node.process_and_update_children(self)?;
                    process_test_results(fid, node, &self)?
                } else {
                    Return::ProcessChildren
                }
            }
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFailed(ids) | FlowCondition::IfAnyFailed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "FAILED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAnySitesFailed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfAnySitesFlag(ids_to_flags(
                            ids, "FAILED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllSitesFailed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfAllSitesFlag(ids_to_flags(
                            ids, "FAILED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllFailed(ids) => {
                    let mut n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfFlag(vec![format!(
                            "{}_FAILED",
                            ids.last().unwrap()
                        )]))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    for (i, id) in ids.into_iter().rev().enumerate() {
                        // The first id is already done above
                        if i != 0 {
                            n = node!(PGM::Condition, FlowCondition::IfFlag(vec![format!("{}_FAILED", id)]) => n);
                        }
                    }
                    Return::Replace(n)
                }
                FlowCondition::IfPassed(ids) | FlowCondition::IfAnyPassed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "PASSED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAnySitesPassed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfAnySitesFlag(ids_to_flags(
                            ids, "PASSED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllSitesPassed(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfAllSitesFlag(ids_to_flags(
                            ids, "PASSED",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfAllPassed(ids) => {
                    let mut n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfFlag(vec![format!(
                            "{}_PASSED",
                            ids.last().unwrap()
                        )]))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    for (i, id) in ids.into_iter().rev().enumerate() {
                        // The first id is already done above
                        if i != 0 {
                            n = node!(PGM::Condition, FlowCondition::IfFlag(vec![format!("{}_PASSED", id)]) => n);
                        }
                    }
                    Return::Replace(n)
                }
                FlowCondition::IfRan(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::IfFlag(ids_to_flags(
                            ids, "RAN",
                        )))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::UnlessRan(ids) => {
                    let n = node.updated(
                        Some(PGM::Condition(FlowCondition::UnlessFlag(ids_to_flags(
                            ids, "RAN",
                        )))),
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
    use crate::prog_gen::{BinType, FlowCondition, FlowID, GroupType, PGM};
    use crate::Result;

    #[test]
    fn it_updates_both_sides_of_the_relationship() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad)
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3))
            ),
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 4, FlowID::from_int(4))
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 4, FlowID::from_int(5))
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnPassed, FlowID::from_int(1) =>
                    node!(PGM::SetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                    node!(PGM::Continue),
                    node!(PGM::SetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGM::OnPassed, FlowID::from_int(2) =>
                    node!(PGM::SetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t1_PASSED".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3))
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t2_PASSED".to_string()]) =>
                node!(PGM::Test, 4, FlowID::from_int(4))
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t2_FAILED".to_string()]) =>
                node!(PGM::Test, 4, FlowID::from_int(5))
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn embedded_test_results_are_processed() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(3)]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::SetFlag, "t1_FAILED".to_string(), true, true),
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::Test, 3, FlowID::from_int(3) =>
                    node!(PGM::OnFailed, FlowID::from_int(3) =>
                        node!(PGM::SetFlag, "t3_FAILED".to_string(), true, true),
                        node!(PGM::Continue),
                    ),
                ),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["t3_FAILED".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn any_failed_is_processed() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::IfAnyFailed(vec![FlowID::from_int(1), FlowID::from_int(2)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::SetFlag, "t1_FAILED".to_string(), true, true),
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::SetFlag, "t2_FAILED".to_string(), true, true),
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t1_FAILED".to_string(), "t2_FAILED".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3))
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn group_based_if_failed_is_processed() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("g1")]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::OnFailed, FlowID::from_str("g1") =>
                    node!(PGM::SetFlag, "g1_FAILED".to_string(), true, true),
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["g1_FAILED".to_string()]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn group_based_if_passed_is_processed() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_str("g1")]) =>
                node!(PGM::Group, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Group, "G1".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g1")) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Test, 2, FlowID::from_int(2)),
                node!(PGM::OnPassed, FlowID::from_str("g1") =>
                    node!(PGM::SetFlag, "g1_PASSED".to_string(), true, true),
                ),
                node!(PGM::OnFailed, FlowID::from_str("g1") =>
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["g1_PASSED".to_string()]) =>
                node!(PGM::Group, "G2".to_string(), None, GroupType::Flow, Some(FlowID::from_str("g2")) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn ran_conditions_are_converted_to_flag_conditions() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::Condition, FlowCondition::UnlessRan(vec![FlowID::from_int(1)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfRan(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::SetFlag, "t1_RAN".to_string(), true, true),
            node!(PGM::Test, 2, FlowID::from_int(2)),
            node!(PGM::SetFlag, "t2_RAN".to_string(), true, true),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["t1_RAN".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t2_RAN".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_add_continue_to_the_parent_test_if_it_is_already_set_to_delayed() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                    node!(PGM::Delayed),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnPassed, FlowID::from_int(1) =>
                    node!(PGM::SetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                    node!(PGM::Delayed),
                    node!(PGM::SetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGM::OnPassed, FlowID::from_int(2) =>
                    node!(PGM::SetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t1_PASSED".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t2_PASSED".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["t2_FAILED".to_string()]) =>
                    node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }

    #[test]
    fn if_any_site_conditions_work() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1)),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfAnySitesPassed(vec![FlowID::from_int(1)]) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfAllSitesPassed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Condition, FlowCondition::IfAnySitesFailed(vec![FlowID::from_int(2)]) =>
                node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
        );

        let expected = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Test, 1, FlowID::from_int(1) =>
                node!(PGM::OnPassed, FlowID::from_int(1) =>
                    node!(PGM::SetFlag, "t1_PASSED".to_string(), true, true),
                ),
                node!(PGM::OnFailed, FlowID::from_int(1) =>
                    node!(PGM::Continue),
                ),
            ),
            node!(PGM::Test, 2, FlowID::from_int(2) =>
                node!(PGM::OnFailed, FlowID::from_int(2) =>
                    node!(PGM::Bin, 10, None, BinType::Bad),
                    node!(PGM::Continue),
                    node!(PGM::SetFlag, "t2_FAILED".to_string(), true, true),
                ),
                node!(PGM::OnPassed, FlowID::from_int(2) =>
                    node!(PGM::SetFlag, "t2_PASSED".to_string(), true, true),
                ),
            ),
            node!(PGM::Condition, FlowCondition::IfAnySitesFlag(vec!["t1_PASSED".to_string()]) =>
                    node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfAllSitesFlag(vec!["t2_PASSED".to_string()]) =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
            node!(PGM::Condition, FlowCondition::IfAnySitesFlag(vec!["t2_FAILED".to_string()]) =>
                    node!(PGM::Test, 5, FlowID::from_int(5)),
            ),
        );

        assert_eq!(expected, run(&input)?);
        Ok(())
    }
}
