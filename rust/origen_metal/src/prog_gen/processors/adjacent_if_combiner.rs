use crate::ast::{Node, Processor, Return};
use crate::prog_gen::expressions::expression_variables;
use crate::prog_gen::{FlowCondition, GroupType, PGM};
use crate::Result;
use std::collections::HashMap;

/// This combines adjacent if flag nodes where the flag is in the opposite state, and likewise
/// adjacent enable and expression conditions
///
/// Input:
///
///    PGMFlow("f1")
///        PGMCondition(IfFlag(["SOME_FLAG"]))
///            PGMTest(1, FlowID("t1"))
///        PGMCondition(UnlessFlag(["SOME_FLAG"]))
///            PGMTest(2, FlowID("t2"))
///        PGMCondition(IfExpr("mode == 6"))
///            PGMTest(3, FlowID("t3"))
///        PGMCondition(UnlessExpr("mode == 6"))
///            PGMTest(4, FlowID("t4"))
///
/// Output:
///
///    PGMFlow("f1")
///        PGMCondition(IfFlag(["SOME_FLAG"]))
///            PGMTest(1, FlowID("t1"))
///            PGMElse
///                PGMTest(2, FlowID("t2"))
///        PGMCondition(IfExpr("mode == 6"))
///            PGMTest(3, FlowID("t3"))
///            PGMElse
///                PGMTest(4, FlowID("t4"))
///
/// See here for an example of the kind of flow level effect it has:
/// https://github.com/Origen-SDK/origen_testers/issues/43
///
pub struct AdjacentIfCombiner {
    /// Flags which tests may change
    volatiles: Vec<String>,
    /// Variables declared as outputs, which tests may change natively
    outputs: Vec<String>,
    /// The names of the test invocations, by their IDs
    test_names: HashMap<usize, String>,
    /// Each flow group and sub-flow has its own scope of suite names, as on SMT8, so a reference
    /// such as t.pass does not name a suite within a flow which the condition calls
    flows_have_own_scope: bool,
}

/// Combines the adjacent opposite conditions, a test whose name is not known may be any test
#[cfg(test)]
pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    run_with_test_names(node, HashMap::new(), false)
}

/// Combines the adjacent opposite conditions, with the names of the test invocations, by their
/// IDs, to tell which tests a condition runs, and whether each flow group and sub-flow has its own
/// scope of suite names
pub fn run_with_test_names(
    node: &Node<PGM>,
    test_names: HashMap<usize, String>,
    flows_have_own_scope: bool,
) -> Result<Node<PGM>> {
    let mut p = AdjacentIfCombiner {
        volatiles: vec![],
        outputs: super::declared_output_variables(node),
        test_names,
        flows_have_own_scope,
    };
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
            // Any node whose children run in sequence within one flow
            PGM::Flow(_)
            | PGM::SubFlow(_, _)
            | PGM::Group(_, _, GroupType::Flow, _)
            | PGM::OnFailed(_)
            | PGM::OnPassed(_)
            | PGM::Loop(..)
            | PGM::Condition(_)
            | PGM::Else => {
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
                if self.is_opposite_flag_states(&n1, &n2)
                    && !has_else(&n1)
                    && !has_else(&n2)
                    && self.is_safe_to_combine(&n1, &n2)
                {
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
                return f1 == f2;
            }
        }
        if let PGM::Condition(FlowCondition::UnlessFlag(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::IfFlag(f2)) = &node2.attrs {
                return f1 == f2;
            }
        }
        if let PGM::Condition(FlowCondition::IfEnable(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::UnlessEnable(f2)) = &node2.attrs {
                return f1 == f2;
            }
        }
        if let PGM::Condition(FlowCondition::UnlessEnable(f1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::IfEnable(f2)) = &node2.attrs {
                return f1 == f2;
            }
        }
        if let PGM::Condition(FlowCondition::IfExpr(e1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::UnlessExpr(e2)) = &node2.attrs {
                return e1 == e2;
            }
        }
        if let PGM::Condition(FlowCondition::UnlessExpr(e1)) = &node1.attrs {
            if let PGM::Condition(FlowCondition::IfExpr(e2)) = &node2.attrs {
                return e1 == e2;
            }
        }
        false
    }

    /// Returns true if the given flag or variable is declared volatile. Flags are rendered upper
    /// case, so a declaration in another case may be the same variable, it is treated as the same
    fn is_volatile(&self, name: &str) -> bool {
        self.volatiles.iter().any(|v| same_name(v, name))
    }

    /// Nodes won't be collapsed if node1 touches the shared run flag, enable or expression
    /// variables, i.e. if there is any chance that by the time it would naturally execute node2,
    /// the condition could have been changed by node1
    fn is_safe_to_combine(&self, node1: &Box<Node<PGM>>, _node2: &Box<Node<PGM>>) -> bool {
        match &node1.attrs {
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFlag(fl)
                | FlowCondition::UnlessFlag(fl)
                | FlowCondition::IfEnable(fl)
                | FlowCondition::UnlessEnable(fl) => {
                    let volatile = fl.iter().any(|flag| {
                        self.is_volatile(flag) || super::flag_reads_output(flag, &self.outputs)
                    });
                    if !volatile || !ContainsATest::run(node1) {
                        return fl.iter().all(|flag| !ChangesFlag::run(node1, flag, true));
                    }
                }
                FlowCondition::IfExpr(expression) | FlowCondition::UnlessExpr(expression) => {
                    let variables = expression_variables(expression);
                    // A test executed by node1 may change a volatile variable
                    if variables
                        .iter()
                        .any(|v| self.is_volatile(v) || self.outputs.contains(v))
                        && ContainsATest::run(node1)
                    {
                        return false;
                    }
                    // Or the result of a test or flow which the expression reads, e.g. t.pass, when
                    // node1 executes it again
                    let objects = referenced_objects(expression);
                    if !objects.is_empty()
                        && ExecutesOneOf::run(
                            node1,
                            &objects,
                            &self.test_names,
                            self.flows_have_own_scope,
                        )
                    {
                        return false;
                    }
                    return variables
                        .iter()
                        .all(|variable| !ChangesFlag::run(node1, variable, false));
                }
                _ => {}
            },
            _ => {}
        }
        false
    }
}

/// Returns true if the given node is a condition which already has an else branch
fn has_else(node: &Node<PGM>) -> bool {
    node.children.iter().any(|n| matches!(n.attrs, PGM::Else))
}

/// Returns true if the given text mentions the given name as an identifier, compared as
/// same_identifier does, e.g. `F = 0;` mentions F, `@ALARM = 0;` mentions $ALARM, and
/// `probe.execute();` mentions probe and execute
fn mentions_identifier(text: &str, name: &str) -> bool {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$' || c == '@'))
        .any(|word| same_identifier(word, name))
}

/// Returns true if the given names may be the same variable as rendered: without the $ or @
/// prefix of a variable, which an expression does not include and a tester may change, e.g.
/// SMT7 renders the flag $Alarm as @Alarm, and compared as same_name does
fn same_identifier(a: &str, b: &str) -> bool {
    let prefix = |c: char| c == '$' || c == '@';
    same_name(a.trim_start_matches(prefix), b.trim_start_matches(prefix))
}

/// Returns true if the given names are the same as flags are rendered, upper case, e.g. straße
/// and STRASSE
fn same_name(a: &str, b: &str) -> bool {
    a == b || a.to_uppercase() == b.to_uppercase()
}

/// Returns true if the given expression reads a reference to an attribute of something, e.g. the
/// result of a test, t.pass, which expression_variables does not return
#[cfg(test)]
fn reads_a_reference(expression: &str) -> bool {
    !referenced_objects(expression).is_empty()
}

/// Returns the names of the things whose attributes the given expression reads, e.g. t for the
/// result of a test, t.pass, which expression_variables does not return
fn referenced_objects(expression: &str) -> Vec<String> {
    let mut objects = vec![];
    let chars: Vec<char> = expression.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' || c == '\'' {
            i += 1;
            while i < chars.len() && chars[i] != c {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
        } else if c.is_ascii_digit() {
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            // As in expression_variables, whitespace may separate the name, the dot and the
            // attribute
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '.' {
                j += 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j].is_ascii_alphabetic() || chars[j] == '_') {
                    objects.push(chars[start..i].iter().collect());
                }
            }
        } else {
            i += 1;
        }
    }
    objects
}

/// Returns true if the given test name may be the given object: the same name, or the name with
/// the _<n> suffix of a suite whose name is used again in its flow, which the SMT7 and SMT8
/// renderers add. A name which only starts with the object's name, e.g. probe_passed for probe,
/// is another test.
fn may_be_suite(name: &str, object: &str) -> bool {
    object == name
        || object
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('_'))
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Returns true if the given flow name, as rendered, may be the given object, compared without
/// regard to case and allowing for the _<n> suffix of a flow instance made unique within its flow
fn may_be_flow(name: &str, object: &str) -> bool {
    let name = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_uppercase();
    let object = object.to_uppercase();
    object == name
        || object
            .strip_prefix(&name)
            .and_then(|rest| rest.strip_prefix('_'))
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Finds whether a node may execute a test or flow with one of the given names: a test, a test
/// whose name is not known, a flow group or sub-flow, or a raw statement or call which mentions
/// it. Where each flow has its own scope of suite names, the content of a flow which the node
/// calls is not searched, other than its result handlers, which run in the calling flow.
struct ExecutesOneOf<'a> {
    objects: &'a [String],
    test_names: &'a HashMap<usize, String>,
    flows_have_own_scope: bool,
    found: bool,
}

impl<'a> ExecutesOneOf<'a> {
    fn run(
        node: &Node<PGM>,
        objects: &'a [String],
        test_names: &'a HashMap<usize, String>,
        flows_have_own_scope: bool,
    ) -> bool {
        let mut p = ExecutesOneOf {
            objects,
            test_names,
            flows_have_own_scope,
            found: false,
        };
        let _ = node.process(&mut p);
        p.found
    }

    fn may_be_a_suite(&self, name: &str) -> bool {
        self.objects.iter().any(|object| may_be_suite(name, object))
    }

    fn may_be_a_flow(&self, name: &str) -> bool {
        self.objects.iter().any(|object| may_be_flow(name, object))
    }
}

impl<'a> Processor<PGM> for ExecutesOneOf<'a> {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        if self.found {
            return Ok(Return::None);
        }
        match &node.attrs {
            PGM::Test(id, _) | PGM::Cz(id, _, _) => match self.test_names.get(id) {
                Some(name) => {
                    if self.may_be_a_suite(name) {
                        self.found = true;
                    }
                }
                // A test whose name is not known may be any test
                None => self.found = true,
            },
            PGM::TestStr(name, _, _, _, _) => {
                if self.may_be_a_suite(name) {
                    self.found = true;
                }
            }
            PGM::SubFlow(name, _) | PGM::Group(name, _, GroupType::Flow, _) => {
                if self.may_be_a_flow(name) {
                    self.found = true;
                } else if self.flows_have_own_scope {
                    for child in &node.children {
                        if matches!(child.attrs, PGM::OnPassed(_) | PGM::OnFailed(_)) {
                            let _ = child.process(self)?;
                        }
                    }
                    return Ok(Return::None);
                }
            }
            PGM::Render(text) => {
                if self.objects.iter().any(|o| mentions_identifier(text, o)) {
                    self.found = true;
                }
            }
            PGM::Call(name, args) | PGM::Callback(name, args) => {
                if std::iter::once(name)
                    .chain(args.iter())
                    .any(|text| self.objects.iter().any(|o| mentions_identifier(text, o)))
                {
                    self.found = true;
                }
            }
            _ => {}
        }
        Ok(Return::ProcessChildren)
    }
}

/// Finds whether a node changes the given flag, enable or flow variable, including a declared
/// output variable, which a test method in the declaring flow may change
struct ChangesFlag<'a> {
    flag: &'a str,
    /// The name is a flag or enable, rendered upper case, rather than an expression variable
    is_flag: bool,
    flag_found: bool,
}

impl<'a> ChangesFlag<'a> {
    fn run(node: &Node<PGM>, flag: &'a str, is_flag: bool) -> bool {
        let mut p = ChangesFlag {
            flag_found: false,
            flag: flag,
            is_flag,
        };
        let _ = node.process(&mut p);
        p.flag_found
    }
}

impl<'a> Processor<PGM> for ChangesFlag<'a> {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            // A raw tester statement or an external call may change a flag or variable which it
            // mentions
            PGM::Render(text) => {
                if mentions_identifier(text, self.flag) {
                    self.flag_found = true;
                }
                Return::None
            }
            PGM::Call(name, args) | PGM::Callback(name, args) => {
                if std::iter::once(name)
                    .chain(args.iter())
                    .any(|text| mentions_identifier(text, self.flag))
                {
                    self.flag_found = true;
                }
                Return::None
            }
            PGM::SetFlag(flag, _, _)
            | PGM::Enable(flag)
            | PGM::Disable(flag)
            | PGM::SetVariable(flag, _) => {
                // Flags and enables are rendered upper case, so a name in another case may be
                // the same variable, it is treated as the same one
                if same_name(flag, self.flag) {
                    self.flag_found = true;
                }
                Return::None
            }
            PGM::OutputVariable(name) => {
                let names = [name.to_string()];
                if (self.is_flag && super::flag_reads_output(self.flag, &names))
                    || name == self.flag
                {
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
            PGM::Test(_, _) | PGM::TestStr(_, _, _, _, _) => {
                self.test_found = true;
                Return::None
            }
            // A raw tester statement or an external call may execute a test, e.g. `run(t);`
            PGM::Render(_) | PGM::Call(_, _) | PGM::Callback(_, _) => {
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
    use crate::prog_gen::{BinType, FlowCondition, FlowID, GroupType, LoopCount, PGM};
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

    #[test]
    fn should_combine_adjacent_nodes_with_the_opposite_expression() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("mode == 6".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("mode == 6".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("count > 0".to_string()) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
            ),
            node!(PGM::Condition, FlowCondition::IfExpr("count > 0".to_string()) =>
                node!(PGM::Test, 4, FlowID::from_int(4)),
            ),
        );

        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("mode == 6".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Else =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("count > 0".to_string()) =>
                node!(PGM::Test, 3, FlowID::from_int(3)),
                node!(PGM::Else =>
                    node!(PGM::Test, 4, FlowID::from_int(4)),
                ),
            ),
        );

        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_combine_adjacent_nodes_with_different_expressions() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("mode == 6".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("mode == 7".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );

        assert_eq!(input, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_combine_if_the_first_node_assigns_a_variable_of_the_expression() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("passed == 0".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::SetVariable, "passed".to_string(), "1".to_string()),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("passed == 0".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );

        assert_eq!(input, run(&input)?);
        Ok(())
    }

    #[test]
    fn should_not_combine_if_the_first_node_calls_a_flow_that_outputs_a_variable_of_the_condition(
    ) -> Result<()> {
        // A test method in the called flow may change the declared variable
        for (if_cond, unless_cond, name) in [
            (
                FlowCondition::IfExpr("mode == 6".to_string()),
                FlowCondition::UnlessExpr("mode == 6".to_string()),
                "mode",
            ),
            (
                FlowCondition::IfFlag(vec!["DEBUG".to_string()]),
                FlowCondition::UnlessFlag(vec!["DEBUG".to_string()]),
                "DEBUG",
            ),
        ] {
            let input = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::Condition, if_cond =>
                    node!(PGM::Group, "setter".to_string(), None, GroupType::Flow, None =>
                        node!(PGM::Test, 1, FlowID::from_int(1)),
                        node!(PGM::OutputVariable, name.to_string()),
                    ),
                ),
                node!(PGM::Condition, unless_cond =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            );

            assert_eq!(input, run(&input)?);
        }
        Ok(())
    }

    #[test]
    fn should_not_combine_if_the_first_node_runs_a_test_and_the_condition_reads_a_declared_output(
    ) -> Result<()> {
        // Declared at flow level, outside both conditions, a test may change it natively
        for (if_cond, unless_cond, name) in [
            (
                FlowCondition::IfExpr("mode == 6".to_string()),
                FlowCondition::UnlessExpr("mode == 6".to_string()),
                "mode",
            ),
            (
                FlowCondition::IfFlag(vec!["DEBUG".to_string()]),
                FlowCondition::UnlessFlag(vec!["DEBUG".to_string()]),
                "DEBUG",
            ),
            // Flags are rendered upper case, this reads the declared DEBUG
            (
                FlowCondition::IfFlag(vec!["debug".to_string()]),
                FlowCondition::UnlessFlag(vec!["debug".to_string()]),
                "DEBUG",
            ),
            // Any operand of a flag condition
            (
                FlowCondition::IfFlag(vec!["STATIC".to_string(), "DEBUG".to_string()]),
                FlowCondition::UnlessFlag(vec!["STATIC".to_string(), "DEBUG".to_string()]),
                "DEBUG",
            ),
        ] {
            let input = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::OutputVariable, name.to_string()),
                node!(PGM::Condition, if_cond.clone() =>
                    node!(PGM::TestStr, "setter".to_string(), FlowID::from_int(1), None, None, None),
                ),
                node!(PGM::Condition, unless_cond.clone() =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            );
            assert_eq!(input, run(&input)?);

            // Without a test in the first branch nothing can change it, so they are combined
            let input = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::OutputVariable, name.to_string()),
                node!(PGM::Condition, if_cond.clone() =>
                    node!(PGM::Bin, 3, None, BinType::Bad),
                ),
                node!(PGM::Condition, unless_cond =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            );
            let output = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::OutputVariable, name.to_string()),
                node!(PGM::Condition, if_cond =>
                    node!(PGM::Bin, 3, None, BinType::Bad),
                    node!(PGM::Else =>
                        node!(PGM::Test, 2, FlowID::from_int(2)),
                    ),
                ),
            );
            assert_eq!(output, run(&input)?);
        }
        Ok(())
    }

    fn opposite_pair() -> Vec<crate::ast::Node<PGM>> {
        vec![
            node!(PGM::Condition, FlowCondition::IfExpr("t.pass".to_string()) =>
                node!(PGM::SetFlag, "FLAG_B".to_string(), true, false),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("t.pass".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        ]
    }

    fn combined_pair() -> crate::ast::Node<PGM> {
        node!(PGM::Condition, FlowCondition::IfExpr("t.pass".to_string()) =>
            node!(PGM::SetFlag, "FLAG_B".to_string(), true, false),
            node!(PGM::Else =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        )
    }

    #[test]
    fn opposite_conditions_are_combined_within_loops_conditions_and_else_branches() -> Result<()> {
        let wrap = |mut node: crate::ast::Node<PGM>, children: Vec<crate::ast::Node<PGM>>| {
            for child in children {
                node.add_child(child);
            }
            node
        };
        let containers = || {
            vec![
                node!(
                    PGM::Loop,
                    Some(LoopCount::Fixed(2)),
                    Some("i".to_string()),
                    false,
                    1
                ),
                node!(
                    PGM::Condition,
                    FlowCondition::IfEnable(vec!["e".to_string()])
                ),
                node!(
                    PGM::Condition,
                    FlowCondition::IfFailed(vec![FlowID::from_int(1)])
                ),
            ]
        };
        for container in containers() {
            let input =
                node!(PGM::Flow, "f1".to_string() => wrap(container.clone(), opposite_pair()));
            let output =
                node!(PGM::Flow, "f1".to_string() => wrap(container, vec![combined_pair()]));
            assert_eq!(output, run(&input)?);
        }

        // Within the else branch of a combined pair
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
            ),
            wrap(
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["F".to_string()])),
                opposite_pair()
            ),
        );
        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Else => combined_pair()),
            ),
        );
        assert_eq!(output, run(&input)?);
        Ok(())
    }

    #[test]
    fn conditions_on_different_lists_of_flags_are_not_combined() -> Result<()> {
        for (a, b) in [
            (vec!["A", "B"], vec!["A", "C"]),
            (vec!["A"], vec!["A", "B"]),
        ] {
            let flags = |v: &Vec<&str>| v.iter().map(|f| f.to_string()).collect::<Vec<_>>();
            let input = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::Condition, FlowCondition::IfFlag(flags(&a)) =>
                    node!(PGM::Test, 1, FlowID::from_int(1)),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(flags(&b)) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            );
            assert_eq!(input, run(&input)?);
        }
        Ok(())
    }

    #[test]
    fn unsafe_pairs_within_a_loop_are_not_combined() -> Result<()> {
        let set_flag =
            |flag: &str, state: bool| node!(PGM::SetFlag, flag.to_string(), state, false);
        let pairs = vec![
            // The first branch changes the flag, named in another case
            vec![
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["ready".to_string()]) =>
                    set_flag("READY", false),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["ready".to_string()]) =>
                    set_flag("SECOND_BRANCH", true),
                ),
            ],
            // The first branch runs a test again, whose result the expression reads
            vec![
                node!(PGM::Condition, FlowCondition::IfExpr("probe.pass".to_string()) =>
                    node!(PGM::TestStr, "probe".to_string(), FlowID::from_str("probe_again"), None, None, None),
                ),
                node!(PGM::Condition, FlowCondition::UnlessExpr("probe.pass".to_string()) =>
                    set_flag("RETEST_FAILED", true),
                ),
            ],
            // The first branch runs the test again with a raw statement
            vec![
                node!(PGM::Condition, FlowCondition::IfExpr("probe.pass".to_string()) =>
                    node!(PGM::Render, "probe.execute();".to_string()),
                ),
                node!(PGM::Condition, FlowCondition::UnlessExpr("probe.pass".to_string()) =>
                    set_flag("RETEST_FAILED", true),
                ),
            ],
            // The first branch may change the flag with a raw statement
            vec![
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                    node!(PGM::Render, "F = 0;".to_string()),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["F".to_string()]) =>
                    set_flag("C", true),
                ),
            ],
            // A spaced reference to the result of a test which the first branch runs again
            vec![
                node!(PGM::Condition, FlowCondition::IfExpr("probe . pass".to_string()) =>
                    node!(PGM::TestStr, "probe".to_string(), FlowID::from_str("probe_again"), None, None, None),
                ),
                node!(PGM::Condition, FlowCondition::UnlessExpr("probe . pass".to_string()) =>
                    set_flag("RETEST_FAILED", true),
                ),
            ],
            // The first condition already has an else branch
            vec![
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                    set_flag("A", true),
                    node!(PGM::Else => set_flag("B", true)),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["F".to_string()]) =>
                    set_flag("C", true),
                ),
            ],
            // The second condition already has an else branch
            vec![
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                    set_flag("A", true),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["F".to_string()]) =>
                    set_flag("C", true),
                    node!(PGM::Else => set_flag("B", true)),
                ),
            ],
        ];
        for pair in pairs {
            let mut looped = node!(
                PGM::Loop,
                Some(LoopCount::Fixed(2)),
                Some("i".to_string()),
                false,
                1
            );
            for node in pair {
                looped.add_child(node);
            }
            let input = node!(PGM::Flow, "f1".to_string() => looped);
            assert_eq!(input, run(&input)?);
        }
        Ok(())
    }

    #[test]
    fn a_volatile_flag_declared_in_another_case_is_volatile() -> Result<()> {
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "READY".to_string()),
            node!(PGM::Loop, Some(LoopCount::Fixed(2)), Some("i".to_string()), false, 1 =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["ready".to_string()]) =>
                    node!(PGM::TestStr, "native_writer".to_string(), FlowID::from_str("native_writer"), None, None, None),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["ready".to_string()]) =>
                    node!(PGM::SetFlag, "SECOND".to_string(), true, false),
                ),
            ),
        );
        assert_eq!(input, run(&input)?);
        Ok(())
    }

    #[test]
    fn references_are_found_in_expressions() {
        use super::reads_a_reference;
        assert!(reads_a_reference("probe.pass"));
        assert!(reads_a_reference("x > 1 && t_1.fail"));
        assert!(reads_a_reference("probe . pass"));
        assert!(reads_a_reference("probe\n.pass"));
        assert!(!reads_a_reference("x > 1.5"));
        assert!(!reads_a_reference("name == \"a.b\""));
        assert!(!reads_a_reference("mode == 6"));
    }

    #[test]
    fn a_raw_statement_which_does_not_mention_the_condition_does_not_stop_combining() -> Result<()>
    {
        let render = |text: &str| node!(PGM::Render, text.to_string());
        // The first branch runs a test and renders a statement, e.g. binning on its failure,
        // which neither runs a test nor mentions the expression's variable
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("MODE == 6".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                render("if ($ALARM == 1) { multi_bin; }"),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("MODE == 6".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );
        let output = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("MODE == 6".to_string()) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                render("if ($ALARM == 1) { multi_bin; }"),
                node!(PGM::Else =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
        );
        assert_eq!(output, run(&input)?);

        // Likewise for a flag condition, and a call
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["F".to_string()]) =>
                render("multi_bin;"),
                node!(PGM::Call, "log_it".to_string(), vec!["x".to_string()]),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["F".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
        );
        assert_eq!(run(&input)?.to_string().matches("Else").count(), 1);
        Ok(())
    }

    #[test]
    fn a_reference_to_a_test_stops_combining_only_when_the_first_branch_runs_that_test(
    ) -> Result<()> {
        use super::run_with_test_names;
        use std::collections::HashMap;
        let pair = |first: crate::ast::Node<PGM>| {
            node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::Condition, FlowCondition::IfExpr("probe.pass".to_string()) => first),
                node!(PGM::Condition, FlowCondition::UnlessExpr("probe.pass".to_string()) =>
                    node!(PGM::SetFlag, "RETEST_FAILED".to_string(), true, false),
                ),
            )
        };
        let combined_on =
            |input: &crate::ast::Node<PGM>, names: HashMap<usize, String>, own_scope: bool| {
                run_with_test_names(input, names, own_scope)
                    .unwrap()
                    .to_string()
                    .contains("Else")
            };
        let combined = |input: &crate::ast::Node<PGM>, names: HashMap<usize, String>| {
            combined_on(input, names, false)
        };
        let names = HashMap::from([
            (1, "other".to_string()),
            (2, "probe".to_string()),
            (4, "probe_passed".to_string()),
            (5, "Probe".to_string()),
            (6, "probe_2".to_string()),
        ]);
        // Another test is run, combined as in 1.11.0
        assert!(combined(
            &pair(node!(PGM::Test, 1, FlowID::from_int(1))),
            names.clone()
        ));
        // Including one whose name only starts with the referenced name, or differs in case
        assert!(combined(
            &pair(node!(PGM::Test, 4, FlowID::from_int(4))),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(PGM::Test, 5, FlowID::from_int(5))),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(PGM::Test, 6, FlowID::from_int(6))),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(
                PGM::TestStr,
                "probe_passed".to_string(),
                FlowID::from_str("pp"),
                None,
                None,
                None
            )),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(PGM::Render, "probe_passed.execute();".to_string())),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(
                PGM::TestStr,
                "other".to_string(),
                FlowID::from_str("o"),
                None,
                None,
                None
            )),
            names.clone()
        ));
        assert!(combined(
            &pair(node!(PGM::Render, "multi_bin;".to_string())),
            names.clone()
        ));
        // The referenced test is run again, not combined
        assert!(!combined(
            &pair(node!(PGM::Test, 2, FlowID::from_int(2))),
            names.clone()
        ));
        assert!(!combined(
            &pair(node!(
                PGM::TestStr,
                "probe".to_string(),
                FlowID::from_str("p"),
                None,
                None,
                None
            )),
            names.clone()
        ));
        assert!(!combined(
            &pair(node!(PGM::Render, "probe.execute();".to_string())),
            names.clone()
        ));
        // Or a flow of that name, or a test whose name is not known
        assert!(!combined(
            &pair(node!(
                PGM::Group,
                "Probe".to_string(),
                None,
                GroupType::Flow,
                Some(FlowID::from_str("g"))
            )),
            names.clone()
        ));
        assert!(!combined(
            &pair(node!(PGM::Test, 3, FlowID::from_int(3))),
            names.clone()
        ));
        // A suite whose name is used again is rendered with a suffix, e.g. probe_1, so a
        // reference to probe_1 may be the test probe
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("probe_1.pass".to_string()) =>
                node!(PGM::Test, 2, FlowID::from_int(2)),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("probe_1.pass".to_string()) =>
                node!(PGM::SetFlag, "RETEST_FAILED".to_string(), true, false),
            ),
        );
        assert!(!combined_on(&input, names.clone(), true));
        assert!(!combined_on(&input, names.clone(), false));
        // A suite of that name within a flow which the first branch calls is the reference's
        // suite only where the flows share one scope of suite names
        let nested = pair(node!(
            PGM::Group,
            "nested".to_string(),
            None,
            GroupType::Flow,
            Some(FlowID::from_str("n"))
            => node!(PGM::Test, 2, FlowID::from_int(2))
        ));
        assert!(combined_on(&nested, names.clone(), true));
        assert!(!combined_on(&nested, names.clone(), false));
        // But the flow's result handlers run in the calling flow
        let handled = pair(node!(
            PGM::Group,
            "nested".to_string(),
            None,
            GroupType::Flow,
            Some(FlowID::from_str("n"))
            => node!(PGM::Test, 1, FlowID::from_int(1)),
               node!(PGM::OnFailed, FlowID::from_str("n") =>
                   node!(PGM::Test, 2, FlowID::from_int(2))
               )
        ));
        assert!(!combined_on(&handled, names, true));
        Ok(())
    }

    #[test]
    fn a_raw_write_of_a_flag_is_found_by_its_rendered_name() -> Result<()> {
        for (flag, statement) in [
            ("$ALARM", "$ALARM = 0;"),
            ("straße", "STRASSE = 0;"),
            // SMT7 renders the flag $Alarm as @Alarm
            ("$Alarm", "@Alarm = 0;"),
        ] {
            let input = node!(PGM::Flow, "f1".to_string() =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec![flag.to_string()]) =>
                    node!(PGM::Render, statement.to_string()),
                ),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec![flag.to_string()]) =>
                    node!(PGM::SetFlag, "B".to_string(), true, false),
                ),
            );
            assert_eq!(input, run(&input)?, "{}", flag);
        }
        // An expression reads a $ variable without its prefix
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("$ALARM == 1".to_string()) =>
                node!(PGM::Render, "$ALARM = 0;".to_string()),
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("$ALARM == 1".to_string()) =>
                node!(PGM::SetFlag, "B".to_string(), true, false),
            ),
        );
        assert_eq!(input, run(&input)?);
        Ok(())
    }

    #[test]
    fn a_raw_statement_may_run_a_test_which_changes_a_volatile_flag() -> Result<()> {
        // e.g. SMT7's run(probe);
        let input = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Volatile, "READY".to_string()),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["READY".to_string()]) =>
                node!(PGM::Render, "run(probe);".to_string()),
            ),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["READY".to_string()]) =>
                node!(PGM::SetFlag, "B".to_string(), true, false),
            ),
        );
        assert_eq!(input, run(&input)?);
        Ok(())
    }
}
