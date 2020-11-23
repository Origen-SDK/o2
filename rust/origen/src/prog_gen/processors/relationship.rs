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
                | FlowCondition::IfAllFailed(ids) => {
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
                | FlowCondition::IfAllPassed(ids) => {
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

impl Processor for Relationship {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMTest(_, fid) => {
                let mut node = node.process_and_update_children(self)?;
                // If this test has a dependent
                if self.test_results.contains_key(fid) {
                    for r in &self.test_results[fid] {
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
                                            n.ensure_node_present(Attrs::PGMContinue);
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
                                            n.ensure_node_present(Attrs::PGMContinue);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            TestResult::Ran => {}
                        }
                    }
                }
                Return::Replace(node)
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFailed(ids) | FlowCondition::IfAnyFailed(ids) => {
                    let flags = ids
                        .iter()
                        .map(|id| format!("{}_FAILED", id.to_str()))
                        .collect();
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(flags))),
                        Some(node.process_and_box_children(self)?),
                        None,
                    );
                    Return::Replace(n)
                }
                FlowCondition::IfPassed(ids) => {
                    let flag = format!("{}_PASSED", ids.first().unwrap().to_str());
                    let n = node.updated(
                        Some(Attrs::PGMCondition(FlowCondition::IfFlag(vec![flag]))),
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
                node!(PGMOnFailed, FlowID::from_int(1) =>
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
}

//
//it "group-based if_passed is processed" do
//group :group1, id: :grp1 do
//  test :test1
//  test :test2
//end
//if_passed :grp1 do
//  group :group2 do
//    test :test3
//    test :test4
//  end
//end
//
//atp.raw.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:group,
//      s(:name, "group1"),
//      s(:id, "grp1"),
//      s(:test,
//        s(:object, "test1")),
//      s(:test,
//        s(:object, "test2"))),
//    s(:if_passed, "grp1",
//      s(:group,
//        s(:name, "group2"),
//        s(:test,
//          s(:object, "test3")),
//        s(:test,
//          s(:object, "test4")))))
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:group,
//      s(:name, "group1"),
//      s(:id, "grp1"),
//      s(:test,
//        s(:object, "test1")),
//      s(:test,
//        s(:object, "test2")),
//      s(:on_pass,
//        s(:set_flag, "grp1_PASSED", "auto_generated")),
//      s(:on_fail,
//        s(:continue))),
//    s(:if_flag, "grp1_PASSED",
//      s(:group,
//        s(:name, "group2"),
//        s(:test,
//          s(:object, "test3")),
//        s(:test,
//          s(:object, "test4")))))
//end
//
//it "ran conditions are converted to flag conditions" do
//test :test1, id: :e1
//test :test2, id: :e2
//test :test3, unless_ran: :e1
//if_ran :e2 do
//  test :test4
//end
//
//atp.raw.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "e1")),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, "e2")),
//    s(:unless_ran, "e1",
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_ran, "e2",
//      s(:test,
//        s(:object, "test4"))))
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "e1")),
//    s(:set_flag, "e1_RAN", "auto_generated"),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, "e2")),
//    s(:set_flag, "e2_RAN", "auto_generated"),
//    s(:unless_flag, "e1_RAN",
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_flag, "e2_RAN",
//      s(:test,
//        s(:object, "test4"))))
//end
//
//it "should not add continue to the parent test if it is already set to :delayed" do
//test :test1, id: :t1
//test :test2, id: :t2, bin: 10, delayed: true
//test :test3, if_passed: :t1
//test :test4, if_passed: :t2
//test :test5, if_failed: :t2
//
//atp.raw.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, :t1)),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, :t2),
//      s(:on_fail,
//        s(:set_result, "fail",
//          s(:bin, 10)),
//        s(:delayed, true))),
//    s(:if_passed, :t1,
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_passed, :t2,
//      s(:test,
//        s(:object, "test4"))),
//    s(:if_failed, :t2,
//      s(:test,
//        s(:object, "test5"))))
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, :t1),
//      s(:on_pass,
//        s(:set_flag, "t1_PASSED", "auto_generated")),
//      s(:on_fail,
//        s(:continue))),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, :t2),
//      s(:on_fail,
//        s(:set_result, "fail",
//          s(:bin, 10)),
//        s(:delayed, true),
//        s(:set_flag, "t2_FAILED", "auto_generated")),
//      s(:on_pass,
//        s(:set_flag, "t2_PASSED", "auto_generated"))),
//    s(:if_flag, "t1_PASSED",
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_flag, "t2_PASSED",
//      s(:test,
//        s(:object, "test4"))),
//    s(:if_flag, "t2_FAILED",
//      s(:test,
//        s(:object, "test5"))))
//end
//
//it "if_any_site conditions work" do
//test :test1, id: :t1
//test :test2, id: :t2, bin: 10
//test :test3, if_any_site_passed: :t1
//test :test4, if_all_sites_passed: :t2
//test :test5, if_any_site_failed: :t2
//
//atp.raw.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "t1")),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, "t2"),
//      s(:on_fail,
//        s(:set_result, "fail",
//          s(:bin, 10)))),
//    s(:if_any_sites_passed, "t1",
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_all_sites_passed, "t2",
//      s(:test,
//        s(:object, "test4"))),
//    s(:if_any_sites_failed, "t2",
//      s(:test,
//        s(:object, "test5"))))
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, :t1),
//      s(:on_pass,
//        s(:set_flag, "t1_PASSED", "auto_generated")),
//      s(:on_fail,
//        s(:continue))),
//    s(:test,
//      s(:object, "test2"),
//      s(:id, :t2),
//      s(:on_fail,
//        s(:set_result, "fail",
//          s(:bin, 10)),
//        s(:continue),
//        s(:set_flag, "t2_FAILED", "auto_generated")),
//      s(:on_pass,
//        s(:set_flag, "t2_PASSED", "auto_generated"))),
//    s(:if_any_sites_flag, "t1_PASSED",
//      s(:test,
//        s(:object, "test3"))),
//    s(:if_all_sites_flag, "t2_PASSED",
//      s(:test,
//        s(:object, "test4"))),
//    s(:if_any_sites_flag, "t2_FAILED",
//      s(:test,
//        s(:object, "test5"))))
//end
//end
