use super::stil;
use super::utility::transaction::Transaction;
use crate::prog_gen::{BinType, FlowCondition, FlowID, GroupType, ParamValue, PatternGroupType};
use crate::services::swd::Acknowledgements;
use crate::testers::SupportedTester;
use indexmap::IndexMap;
use std::collections::HashMap;

pub type Id = usize;
type Metadata = Option<IndexMap<String, crate::Metadata>>;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Attrs {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,
    // A simple node that is quick to write and use in processor unit tests
    T(usize),

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Data Types
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Integer(i64),
    Float(f64),
    String(String),
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Common pat gen and prog gen nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    TesterEq(Vec<SupportedTester>), // Child nodes should only be processed when targetting the given tester(s)
    TesterNeq(Vec<SupportedTester>), // Child nodes should only be processed unless targetting the given tester(s)
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Test (pat gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Test(String),
    Comment(u8, String), // level, msg
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Timeset nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SetTimeset(usize), // Indicates both a set or change of the current timeset
    ClearTimeset,
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Pinheader nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SetPinHeader(usize), // Indicates the pin header selected
    ClearPinHeader,
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Pattern generation nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    PinGroupAction(usize, Vec<String>, Option<HashMap<String, crate::Metadata>>),
    PinAction(usize, String, Option<HashMap<String, crate::Metadata>>),
    Capture(crate::Capture, Metadata),
    EndCapture(Option<usize>),
    Overlay(crate::Overlay, Metadata),
    EndOverlay(Option<usize>),
    Opcode(String, IndexMap<String, String>), // Opcode, Arguments<Argument Key, Argument Value>
    Cycle(u32, bool),                         // repeat (0 not allowed), compressable
    PatternHeader,
    PatternEnd, // Represents the end of a pattern. Note: this doesn't necessarily need to be the last node, but
    // represents the end of the 'pattern vectors', for vector-based testers.
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Register transaction nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    RegWrite(Transaction), // Id, BigUint, Option<BigUint>, Option<String>), // reg_id, data, overlay_enable, overlay_str
    RegVerify(
        Transaction, // Id,
                     // BigUint,
                     // Option<BigUint>,
                     // Option<BigUint>,
                     // Option<BigUint>,
                     // Option<String>
    ), // reg_id, data, verify_enable, capture_enable, overlay_enable, overlay_str
    RegCapture(Transaction),
    RegOverlay(Transaction),
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// JTAG nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    JTAGWriteIR(
        Id, // JTAG ID
        Transaction,
        Metadata,
    ),
    JTAGVerifyIR(
        Id, // JTAG ID
        Transaction,
        Metadata,
    ),
    JTAGWriteDR(
        Id, // JTAG ID
        Transaction,
        Metadata,
    ),
    JTAGVerifyDR(
        Id, // JTAG ID
        Transaction,
        Metadata,
    ),
    JTAGReset(Id, Metadata),
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// SWD nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SWDWriteAP(
        Id, // SWD ID
        Transaction,
        Acknowledgements,
        Metadata,
    ),
    SWDVerifyAP(
        Id, // SWD ID
        Transaction,
        Acknowledgements, // SWD Acknowledgement
        Option<bool>,     // Parity Compare
        Metadata,
    ),
    SWDWriteDP(
        Id, // SWD ID
        Transaction,
        Acknowledgements, // SWD Acknowledgement
        Metadata,
    ),
    SWDVerifyDP(
        Id, // SWD ID
        Transaction,
        Acknowledgements, // SWD Acknowledgement
        Option<bool>,     // Parity Compare
        Metadata,
    ),
    SWDLineReset,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Arm Debug's JTAG DP Nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    JTAGDPWriteDP(
        Id, // JTAG DP Id
        Transaction,
        Metadata,
    ),
    JTAGDPVerifyDP(Id, Transaction, Metadata),
    JTAGDPWriteAP(Id, Transaction, Metadata),
    JTAGDPVerifyAP(Id, Transaction, Metadata),

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Arm Debug nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    ArmDebugMemAPWriteReg(
        Id,    // MemAP Id
        usize, // MemAP address
        Transaction,
        Metadata,
    ),
    ArmDebugMemAPWriteInternalReg(
        Id,    // MemAP Id
        usize, // MemAP address
        Transaction,
        Metadata,
    ),
    ArmDebugMemAPVerifyReg(
        Id,    // MemAP ID
        usize, // MemAP address
        Transaction,
        Metadata,
    ),
    ArmDebugMemAPVerifyInternalReg(
        Id,    // MemAP ID
        usize, // MemAP address
        Transaction,
        Metadata,
    ),
    ArmDebugWriteDP(
        Id, // DP ID
        Transaction,
        Metadata,
    ),
    ArmDebugVerifyDP(
        Id, // DP ID
        Transaction,
        Metadata,
    ),
    ArmDebugSwjJTAGToSWD(Id), // arm_debug_id - Switch DP from JTAG to SWD
    ArmDebugSwjSWDToJTAG(Id), // arm_debug_id - Switch DP from SWD to JTAG
    // ArmDebugSWJ__EnterDormant, // Switch DP to dormant
    // ArmDebugSWJ__ExitDormant, // Switch DP from dormant back to whatever it was prior to entering dormant.

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Simple (Dummy) Protocol nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SimpleProtocolReset(Id),
    SimpleProtocolWrite(Id, Transaction),
    SimpleProtocolVerify(Id, Transaction),

    //// Text (Comment) nodes
    //// Useful for formatting comment blocks in the AST.
    TextSection(Option<String>, Option<u8>), // The start of a new section.
    TextBoundaryLine, // Inserts a 'boundary'. This will be resolve to a line of '*'
    // How exactly this will look in the output is up to the render, but there should be some sort of
    // delimiter or otherwise obvious 'break' in the text
    // This node optionally accepts a 'title', which can be handled however the renderer sees fit.
    // It also optionally accepts a 'level', which the renderer can use to decide how to delimit it
    TextLine, // Content that should appear on the same line. This is only a single node so that other nodes can be used in its children.
    // For example:
    //   TextLine
    //     Text("Hi ")
    //     Author
    //     Text("!")
    // Should render something like: Hi coreyeng!
    // Note: nested TextLines are not supported and exact output is dependent on the renderer.
    Text(String),

    //// Content Nodes
    User,                  // Inserts the current user
    OrigenCommand(String), // The origen command being executed
    Timestamp,             // Inserts a timestamp
    Mode,                  // Inserts the current mode
    TargetsStacked,        // Inserts the current targets as several "Text" nodes
    // TargetsLinearized, // Inserts the targets as a comma-separated list
    OS, // Inserts the OS
    // AppVersion, <- Currently not supported
    AppRoot,
    OrigenVersion,
    OrigenRoot,

    //// Teradyne custom nodes

    //// Advantest custom nodes

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Flow (prog gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    PGMFlow(String),
    PGMSubFlow(String),
    /// Defines a new test, this must be done before attributes can be set via PGMSetAttr. Note that this doesn't
    /// actually add it to the test flow, that must be done via a PGMTest node
    ///         ID,    Name,     Tester,       Library  Template
    PGMDefTest(usize, String, SupportedTester, String, String),
    /// Defines a new test invocation, this must be done before attributes can be set via PGMSetAttr. Note that
    /// this doesn't actually add it to the test flow, that must be done via a PGMTest node
    ///            ID,    Name,     Tester
    PGMDefTestInv(usize, String, SupportedTester),
    /// Assign an existing test to an existing invocation
    ///                 InvID  TestID
    PGMAssignTestToInv(usize, usize),
    /// Set the attribute with the given name within the given test (ID), to the given value
    PGMSetAttr(usize, String, ParamValue),
    /// Execute a test (or invocation) from the flow
    PGMTest(usize, FlowID),
    /// Execute a test (or invocation) from the flow, where the test is simply a string to be inserted
    /// into the flow
    PGMTestStr(String, FlowID),
    /// Defines a new pattern group, also used to model IG-XL pattern sets
    PGMPatternGroup(usize, String, SupportedTester, Option<PatternGroupType>),
    /// Push a pattern to the given pattern group ID
    PGMPushPattern(usize, String, Option<String>),
    /// Render the given text directly to the flow
    PGMRender(String),
    /// Add a log line to the flow
    PGMLog(String),
    /// A FlowID will always be present when the group type is a flow group
    PGMGroup(String, Option<SupportedTester>, GroupType, Option<FlowID>),
    /// All children will be gated by the given condition (if_failed, if_enabled, etc.)
    PGMCondition(FlowCondition),
    /// Execute a test (or invocation) from the flow with a CZ setup reference
    PGMCz(usize, String, FlowID),
    /// Bin (number, is_soft, type, description, priority)
    PGMDefBin(usize, bool, BinType, Option<String>, Option<usize>),
    /// Bin out (hard, soft, type)
    PGMBin(usize, Option<usize>, BinType),
    /// Events to run if the test or group with the given ID failed
    PGMOnFailed(FlowID),
    /// Events to run if the test or group with the given ID passed
    PGMOnPassed(FlowID),

    PGMIGXLSetWaitFlags(usize, Vec<String>),

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// STIL
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    STIL,
    STILUnknown,
    STILVersion(u32, u32), // major, minor
    STILHeader,
    STILTitle(String),
    STILDate(String),
    STILSource(String),
    STILHistory,
    STILAnnotation(String),
    STILInclude(String, Option<String>),
    STILSignals,
    STILSignal(String, stil::SignalType), // name, type
    STILTermination(stil::Termination),
    STILDefaultState(stil::State),
    STILBase(stil::Base, String),
    STILAlignment(stil::Alignment),
    STILScanIn(u32),
    STILScanOut(u32),
    STILDataBitCount(u32),
    STILSignalGroups(Option<String>),
    STILSignalGroup(String),
    STILSigRefExpr,
    STILTimeExpr,
    STILSIUnit(String),
    STILEngPrefix(String),
    STILAdd,
    STILSubtract,
    STILMultiply,
    STILDivide,
    STILParens,
    STILNumberWithUnit,
    STILPatternExec(Option<String>),
    STILCategoryRef(String),
    STILSelectorRef(String),
    STILTimingRef(String),
    STILPatternBurstRef(String),
    STILPatternBurst(String),
    STILSignalGroupsRef(String),
    STILMacroDefs(String),
    STILProcedures(String),
    STILScanStructuresRef(String),
    STILStart(String),
    STILStop(String),
    STILTerminations,
    STILTerminationItem,
    STILPatList,
    STILPat(String),
    STILLabel(String),
    STILTiming(Option<String>),
    STILWaveformTable(String),
    STILPeriod,
    STILInherit(String),
    STILSubWaveforms,
    STILSubWaveform,
    STILWaveforms,
    STILWaveform,
    STILWFChar(String),
    STILEvent,
    STILEventList(Vec<char>),
    STILSpec(Option<String>),
    STILCategory(String),
    STILSpecItem,
    STILTypicalVar(String),
    STILSpecVar(String),
    STILSpecVarItem(stil::Selector),
    STILVariable(String),
    STILSelector(String),
    STILSelectorItem(String, stil::Selector),
    STILScanStructures(Option<String>),
    STILScanChain(String),
    STILScanLength(u64),
    STILScanOutLength(u64),
    STILScanCells,
    STILScanMasterClock,
    STILScanSlaveClock,
    STILScanInversion(u8),
    STILScanInName(String),
    STILScanOutName(String),
    STILNot,
    STILPattern(String),
    STILTimeUnit,
    STILVector,
    STILCyclizedData,
    STILNonCyclizedData,
    STILRepeat(u64),
    STILWaveformFormat,
    STILHexFormat(Option<String>),
    STILDecFormat(Option<String>),
    STILData(String),
    STILTimeValue(u64),
    STILWaveformRef(String),
    STILCondition,
    STILCall(String),
    STILMacro(String),
    STILLoop(u64),
    STILMatchLoop(Option<u64>),
    STILGoto(String),
    STILBreakPoint,
    STILIDDQ,
    STILStopStatement,
}

impl std::fmt::Display for Attrs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Attrs::PinGroupAction(grp_id, _actions, _metadata) => {
                let dut = crate::dut();
                write!(
                    f,
                    "{}",
                    format!("{:?} -> ({})", self, &dut.pin_groups[*grp_id].name)
                )
            }
            Attrs::PinAction(id, _actions, _metadata) => {
                let dut = crate::dut();
                write!(f, "{}", format!("{:?} -> ({})", self, &dut.pins[*id].name))
            }
            _ => write!(f, "{}", format!("{:?}", self)),
        }
    }
}
