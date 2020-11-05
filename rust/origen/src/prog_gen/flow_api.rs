use super::ParamValue;
use super::PatternGroupType;
use crate::generator::ast::Meta;
use crate::testers::SupportedTester;
use crate::{Result, FLOW};

/// Start a sub-flow, the returned reference should be retained and passed to end_sub_flow
pub fn start_sub_flow(name: &str, meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMSubFlow, name.to_owned(); meta);
    FLOW.push_and_open(n)
}

/// End of a sub-flow
pub fn end_sub_flow(ref_id: usize) -> Result<()> {
    FLOW.close(ref_id)
}

/// Defines a new test in the AST, returning its ID.
/// A test must be initially defined before attributes can be set on it. It won't actually
/// appear in the test flow until it is added to if via add_test()
pub fn define_test(
    name: &str,
    tester: &SupportedTester,
    lib_name: &str,
    template_name: &str,
    meta: Option<Meta>,
) -> Result<usize> {
    let id = crate::STATUS.generate_unique_id();
    let n = node!(PGMDefTest, id, name.to_owned(), tester.to_owned(), lib_name.to_owned(), template_name.to_owned(); meta);
    FLOW.push(n)?;
    Ok(id)
}

/// Defines a new test invocation in the AST, returning its ID.
/// A test invoration must be initially defined before attributes can be set on it. It won't actually
/// appear in the test flow until it is added to if via add_test()
pub fn define_test_invocation(
    name: &str,
    tester: &SupportedTester,
    meta: Option<Meta>,
) -> Result<usize> {
    let id = crate::STATUS.generate_unique_id();
    let n = node!(PGMDefTestInv, id, name.to_owned(), tester.to_owned(); meta);
    FLOW.push(n)?;
    Ok(id)
}

/// Set an attribute of either a test or a test invocation
pub fn set_test_attr(id: usize, name: &str, value: ParamValue, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMSetAttr, id, name.to_owned(), value; meta);
    FLOW.push(n)?;
    Ok(())
}

pub fn assign_test_to_invocation(
    invocation_id: usize,
    test_id: usize,
    meta: Option<Meta>,
) -> Result<()> {
    let n = node!(PGMAssignTestToInv, invocation_id, test_id; meta);
    FLOW.push(n)?;
    Ok(())
}

/// Execute the given test (or invocation) from the current flow
pub fn execute_test(id: usize, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMTest, id; meta);
    FLOW.push(n)
}

/// Execute the given test (or invocation) from the current flow, where the test is a string that
/// will be rendered verbatim to the flow - no linkage to an actual test object will be checked or
/// inserted by Origen
pub fn execute_test_str(name: String, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMTestStr, name; meta);
    FLOW.push(n)
}

pub fn define_pattern_group(
    name: String,
    tester: SupportedTester,
    kind: Option<PatternGroupType>,
    meta: Option<Meta>,
) -> Result<usize> {
    let id = crate::STATUS.generate_unique_id();
    let n = node!(PGMPatternGroup, id, name, tester, kind; meta);
    FLOW.push(n)?;
    Ok(id)
}

pub fn push_pattern_to_group(
    id: usize,
    path: String,
    start_label: Option<String>,
    meta: Option<Meta>,
) -> Result<()> {
    let n = node!(PGMPushPattern, id, path, start_label; meta);
    FLOW.push(n)
}
