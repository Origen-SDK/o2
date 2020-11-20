use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowID;
use crate::prog_gen::{FlowCondition, GroupType};
use std::borrow::Cow;
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
    run_flag_table: HashMap<String, usize>,
    nodes_to_inline: Vec<Node>,
    volatile_flags: Vec<String>,
    run_flag_to_remove: Vec<String>,
}

pub fn run(node: &Node, optimize_when_continue: Option<bool>) -> Result<Node> {
    let optimize_when_continue = match optimize_when_continue {
        Some(x) => x,
        None => true,
    };
    let mut p = ExtractRunFlagTable {
        results: HashMap::new(),
    };
    let _ = node.process(&mut p)?;

    let mut p = FlagOptimizer {
        optimize_when_continue: optimize_when_continue,
        run_flag_table: p.results,
        nodes_to_inline: vec![],
        volatile_flags: vec![],
        run_flag_to_remove: vec![],
    };
    let ast = node.process(&mut p)?.unwrap();

    Ok(ast)
}

pub struct ExtractRunFlagTable {
    results: HashMap<String, usize>,
}

/// Extracts the IDs of all tests which have dependents on whether they passed, failed or ran
impl Processor for ExtractRunFlagTable {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFlag(ids) | FlowCondition::UnlessFlag(ids) => {
                    for id in ids {
                        if self.results.contains_key(id) {
                            let x = self.results[id];
                            self.results.insert(id.clone(), x + 1);
                        } else {
                            self.results.insert(id.clone(), 1);
                        }
                    }
                    Return::ProcessChildren
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}

impl Processor for FlagOptimizer {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMVolatile(flag) => {
                self.volatile_flags.push(flag.to_owned());
                Return::Unmodified
            }
            Attrs::PGMGroup(_name, _, kind, _fid) => match kind {
                GroupType::Flow => {
                    Return::Replace(node.updated(None, Some(self.optimize(&node.children)?), None))
                }
                _ => Return::ProcessChildren,
            },
            Attrs::PGMFlow(_)
            | Attrs::PGMSubFlow(_)
            | Attrs::PGMElse
            | Attrs::PGMWhenever
            | Attrs::PGMWheneverAny
            | Attrs::PGMWheneverAll => {
                Return::Replace(node.updated(None, Some(self.optimize(&node.children)?), None))
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::UnlessFlag(_) => {
                    Return::Replace(node.updated(None, Some(self.optimize(&node.children)?), None))
                }
                FlowCondition::IfFlag(ids) => {
                    if let Some(f) = self.run_flag_to_remove.last() {
                        if f == &ids[0] {
                            return Ok(Return::InlineBoxed(self.optimize(&node.children)?));
                        }
                    }
                    Return::Replace(node.updated(None, Some(self.optimize(&node.children)?), None))
                }
                _ => Return::ProcessChildren,
            },
            Attrs::PGMOnFailed(_) => {
                if let Some(to_inline) = self.nodes_to_inline.last() {}
                Return::ProcessChildren // Temporary
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl FlagOptimizer {
    fn optimize(&mut self, nodes: &Vec<Box<Node>>) -> Result<Vec<Box<Node>>> {
        let mut results: Vec<Box<Node>> = vec![];
        let mut node1: Option<Cow<Box<Node>>> = None;
        for node2 in nodes {
            let n2 = Cow::Borrowed(node2);
            if let Some(n1) = node1 {
                if self.can_be_combined(&n1, &n2) {
                    node1 = Some(Cow::Owned(self.combine(n1, n2)?));
                } else {
                    results.push(n1.into_owned());
                    node1 = Some(n2);
                }
            } else {
                node1 = Some(n2);
            }
        }
        Ok(results)
    }

    fn can_be_combined(&self, node1: &Cow<Box<Node>>, node2: &Cow<Box<Node>>) -> bool {
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
                        *val &&
                        is_gated_by_set(name, node2) &&   // The flag set by node1 is gating node2
                        *auto_generated &&                      // The flag was no specified by the user
                        !name.ends_with("_RAN") &&    // Don't compress RAN flags since they will be set by both pass and fail
                        !self.volatile_flags.contains(name) // And finally keep all volatile flags
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

    fn combine(&mut self, node1: Cow<Box<Node>>, node2: Cow<Box<Node>>) -> Result<Box<Node>> {
        self.nodes_to_inline.push(*node2.into_owned());
        let node1 = node1.process_and_update_children(self)?;
        self.nodes_to_inline.pop();
        Ok(Box::new(node1))
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
    use crate::prog_gen::{BinType, FlowCondition, FlowID};
    use crate::Result;

    #[test]
    fn embedded_test_results_are_processed() -> Result<()> {
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
}

//it "works at the top-level" do
//test :test1, id: :t1
//test :test2, if_failed: :t1
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "t1"),
//      s(:on_fail,
//        s(:continue),
//        s(:test,
//          s(:object, "test2")))))
//end
//
//it "doesn't eliminate flags with later references" do
//test :test1, id: :t1
//test :test2, if_failed: :t1
//test :test3
//test :test4, if_failed: :t1
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "t1"),
//      s(:on_fail,
//        s(:set_flag, "t1_FAILED", "auto_generated"),
//        s(:continue),
//        s(:test,
//          s(:object, "test2")))),
//    s(:test,
//      s(:object, "test3")),
//    s(:if_flag, "t1_FAILED",
//      s(:test,
//        s(:object, "test4"))))
//end
//
//it "applies the optimization within nested groups" do
//group :group1 do
//  test :test1, id: :t1
//  test :test2, if_failed: :t1
//end
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:group,
//      s(:name, "group1"),
//      s(:test,
//        s(:object, "test1"),
//        s(:id, "t1"),
//        s(:on_fail,
//          s(:continue),
//          s(:test,
//            s(:object, "test2"))))))
//end
//
//it "a more complex test case with both pass and fail branches to be optimized" do
//test :test1, id: :t1, number: 0
//test :test2, if_passed: :t1, number: 0
//test :test3, if_failed: :t1, number: 0
//bin 10, if_failed: :t1
//
//ast.should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:number, 0),
//      s(:id, "t1"),
//      s(:on_pass,
//        s(:test,
//          s(:object, "test2"),
//          s(:number, 0))),
//      s(:on_fail,
//        s(:continue),
//        s(:test,
//          s(:object, "test3"),
//          s(:number, 0)),
//        s(:set_result, "fail",
//          s(:bin, 10)))))
//end
//
//it "optionally doesn't eliminate flags on tests with a continue" do
//test :test1, id: :t1
//test :test2, if_failed: :t1
//
//ast(optimize_flags_when_continue: false).should ==
//  s(:flow,
//    s(:name, "sort1"),
//    s(:test,
//      s(:object, "test1"),
//      s(:id, "t1"),
//      s(:on_fail,
//        s(:set_flag, "t1_FAILED", "auto_generated"),
//        s(:continue))),
//    s(:if_flag, "t1_FAILED",
//      s(:test,
//        s(:object, "test2"))))
//end
