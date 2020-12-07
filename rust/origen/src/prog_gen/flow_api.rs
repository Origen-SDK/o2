use super::ParamValue;
use super::{BinType, FlowCondition, FlowID, GroupType, Limit, LimitSelector, PatternGroupType};
use crate::generator::ast::Meta;
use crate::testers::SupportedTester;
use crate::{Result, FLOW};

/// Start a sub-flow, the returned reference should be retained and passed to end_block
pub fn start_sub_flow(name: &str, flow_id: Option<FlowID>, meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMSubFlow, name.to_owned(), flow_id; meta);
    FLOW.push_and_open(n)
}

pub fn end_block(ref_id: usize) -> Result<()> {
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
pub fn set_test_attr(
    id: usize,
    name: &str,
    value: Option<ParamValue>,
    meta: Option<Meta>,
) -> Result<()> {
    let n = node!(PGMSetAttr, id, name.to_owned(), value; meta);
    FLOW.push(n)?;
    Ok(())
}

pub fn set_test_limit(
    test_id: Option<usize>,
    inv_id: Option<usize>,
    hilo: LimitSelector,
    value: Option<Limit>,
    meta: Option<Meta>,
) -> Result<()> {
    if test_id.is_none() && inv_id.is_none() {
        return error!("Either a test ID or an invocation ID must be supplied to set_test_limit");
    }
    if test_id.is_some() && inv_id.is_some() {
        return error!("Either a test ID *OR* an invocation ID must be supplied to set_test_limit, but not both");
    }
    let n = node!(PGMSetLimit, test_id, inv_id, hilo, value; meta);
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
pub fn execute_test(id: usize, flow_id: FlowID, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMTest, id, flow_id; meta);
    FLOW.push(n)
}

/// Execute the given test (or invocation) from the current flow, where the test is a string that
/// will be rendered verbatim to the flow - no linkage to an actual test object will be checked or
/// inserted by Origen
pub fn execute_test_str(name: String, flow_id: FlowID, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMTestStr, name, flow_id; meta);
    FLOW.push(n)
}

/// Cz the given test (or invocation) from the current flow
pub fn execute_cz_test(
    id: usize,
    cz_setup: String,
    flow_id: FlowID,
    meta: Option<Meta>,
) -> Result<()> {
    let n = node!(PGMCz, id, cz_setup, flow_id; meta);
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

/// Renders the given string directly to the test flow
pub fn render(text: String, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMRender, text; meta);
    FLOW.push(n)
}

pub fn log(text: String, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMLog, text; meta);
    FLOW.push(n)
}

/// [IGXL only] Set the given wait flags on the given test instance
pub fn set_wait_flags(ti_id: usize, flags: Vec<String>, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMIGXLSetWaitFlags, ti_id, flags; meta);
    FLOW.push(n)
}

/// Used to model flow groups, IG-XL test instance groups, etc.
pub fn start_group(
    name: String,
    tester: Option<SupportedTester>,
    kind: GroupType,
    flow_id: Option<FlowID>,
    meta: Option<Meta>,
) -> Result<usize> {
    if kind == GroupType::Flow && flow_id.is_none() {
        return error!("A flow_id must be supplied when starting a flow group");
    }
    let n = node!(PGMGroup, name, tester, kind, flow_id; meta);
    FLOW.push_and_open(n)
}

pub fn start_condition(condition: FlowCondition, meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMCondition, condition; meta);
    FLOW.push_and_open(n)
}

pub fn define_bin(
    number: usize,
    is_soft: bool,
    kind: BinType,
    description: Option<String>,
    priority: Option<usize>,
    meta: Option<Meta>,
) -> Result<()> {
    let n = node!(PGMDefBin, number, is_soft, kind, description, priority; meta);
    FLOW.push(n)
}

/// Bin out the DUT
pub fn bin(hard: usize, soft: Option<usize>, kind: BinType, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMBin, hard, soft, kind; meta);
    FLOW.push(n)
}

/// Start an on-failed block (events to run if the given test or group failed), the returned
/// reference should be retained and passed to end_block
pub fn start_on_failed(flow_id: FlowID, meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMOnFailed, flow_id; meta);
    FLOW.push_and_open(n)
}

/// Start an on-passed block (events to run if the given test or group passed), the returned
/// reference should be retained and passed to end_block
pub fn start_on_passed(flow_id: FlowID, meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMOnPassed, flow_id; meta);
    FLOW.push_and_open(n)
}

/// Start a resources block, contained tests will not appear in the flow but the test definitions
/// will appear in the generated test program (e.g. in the test instances sheet)
pub fn start_resources(meta: Option<Meta>) -> Result<usize> {
    let n = node!(PGMResources; meta);
    FLOW.push_and_open(n)
}

pub fn set_flag(name: String, state: bool, meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMSetFlag, name, state, false; meta);
    FLOW.push(n)
}

pub fn continue_on_fail(meta: Option<Meta>) -> Result<()> {
    let n = node!(PGMContinue; meta);
    FLOW.push(n)
}
