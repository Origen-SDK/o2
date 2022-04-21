use crate::prog_gen::{
    BinType, FlowCondition, FlowID, GroupType, Limit, LimitSelector, ParamValue, PatternGroupType,
    ResourcesType, UniquenessOption,
};
use crate::testers::SupportedTester;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum PGM {
    /// This will be ignored by all processors, so can be used to indicate the absence of a node
    Nil,
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Common pat gen and prog gen nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    TesterEq(Vec<SupportedTester>), // Child nodes should only be processed when targetting the given tester(s)
    TesterNeq(Vec<SupportedTester>), // Child nodes should only be processed unless targetting the given tester(s)
    //// Teradyne custom nodes

    //// Advantest custom nodes

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Flow (prog gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Flow(String),
    SubFlow(String, Option<FlowID>),
    /// Defines a new test, this must be done before attributes can be set via PGMSetAttr. Note that this doesn't
    /// actually add it to the test flow, that must be done via a PGMTest node
    ///         ID,    Name,     Tester,       Library  Template
    DefTest(usize, String, SupportedTester, String, String),
    /// Defines a new test invocation, this must be done before attributes can be set via PGMSetAttr. Note that
    /// this doesn't actually add it to the test flow, that must be done via a PGMTest node
    ///            ID,    Name,     Tester
    DefTestInv(usize, String, SupportedTester),
    /// Assign an existing test to an existing invocation
    ///                 InvID  TestID
    AssignTestToInv(usize, usize),
    /// Set the attribute with the given name within the given test (ID), to the given value
    SetAttr(usize, String, Option<ParamValue>),
    /// Set the limit of the given test or invocation, (test_id, inv_id, hi/lo, value). Note that either test_id
    /// or inv_id must be present, but not both.
    SetLimit(Option<usize>, Option<usize>, LimitSelector, Option<Limit>),
    /// Execute a test (or invocation) from the flow
    Test(usize, FlowID),
    /// Execute a test (or invocation) from the flow, where the test is simply a string to be inserted
    /// into the flow
    TestStr(String, FlowID),
    /// Defines a new pattern group, also used to model IG-XL pattern sets
    PatternGroup(usize, String, SupportedTester, Option<PatternGroupType>),
    /// Push a pattern to the given pattern group ID
    PushPattern(usize, String, Option<String>),
    /// Render the given text directly to the flow
    Render(String),
    /// Add a log line to the flow
    Log(String),
    /// A FlowID will always be present when the group type is a flow group
    Group(String, Option<SupportedTester>, GroupType, Option<FlowID>),
    /// All children will be gated by the given condition (if_failed, if_enabled, etc.)
    Condition(FlowCondition),
    /// Execute a test (or invocation) from the flow with a CZ setup reference
    Cz(usize, String, FlowID),
    /// Bin (number, is_soft, type, description, priority)
    DefBin(usize, bool, BinType, Option<String>, Option<usize>),
    /// Bin out (hard, soft, type)
    Bin(usize, Option<usize>, BinType),
    /// Events to run if the test or group with the given ID failed
    OnFailed(FlowID),
    /// Events to run if the test or group with the given ID passed
    OnPassed(FlowID),
    /// Any tests contained within a Resources block will not be added to the flow, but will be added
    /// to 'resource' sheets/files, e.g. the test instances sheet
    Resources,
    /// Volatile flag definition (a flag that can be changed state by tests)
    Volatile(String),
    /// Set the flag to the given state (name, state, autogenerated). Autogenerated should be set to differentiate
    /// a flag operation that has been generated by Origen (to implement flow control logic) vs. once that has
    /// been directly specified by the user
    SetFlag(String, bool, bool),
    SetDefaultFlagState(String, bool),
    /// Continue in the event of a failure
    Continue,
    /// Delay binning in the event of a failure
    Delayed,
    Else,
    Whenever,    // Placeholder
    WheneverAny, // Placeholder
    WheneverAll, // Placeholder
    /// Enable a flow switch
    Enable(String),
    /// Disable a flow switch
    Disable(String),
    ResourcesFilename(String, ResourcesType),
    BypassSubFlows,
    FlowDescription(String),
    /// Apply the given uniqueness option to all contained test names, etc.
    Uniqueness(UniquenessOption),

    IGXLSetWaitFlags(usize, Vec<String>),
}

impl std::fmt::Display for PGM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            _ => write!(f, "{}", format!("{:?}", self)),
        }
    }
}