use super::stil;
use crate::core::model::pins::pin::PinActions;
use crate::services::swd::Acknowledgements;
use crate::standard_sub_blocks::arm_debug::ArmDebug;
use crate::standard_sub_blocks::arm_debug::mem_ap::MemAP;
use num_bigint::BigUint;
use std::collections::HashMap;
use indexmap::IndexMap;

type Id = usize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Attrs {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Data Types
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Integer(i64),
    Float(f64),
    String(String),
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Test (pat gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Test(String),
    Comment(u8, String), // level, msg
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Timeset nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SetTimeset(usize),   // Indicates both a set or change of the current timeset
    ClearTimeset,
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Pinheader nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SetPinHeader(usize), // Indicates the pin header selected
    ClearPinHeader,
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Pattern generation nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    PinAction(HashMap<String, (PinActions, u8)>), // Pin IDs, (PinActions, Pin Data)
    Opcode(String, IndexMap<String, String>), // Opcode, Arguments<Argument Key, Argument Value>
    Cycle(u32, bool), // repeat (0 not allowed), compressable
    PatternHeader,
    PatternEnd, // Represents the end of a pattern. Note: this doesn't necessarily need to be the last node, but
                // represents the end of the 'pattern vectors', for vector-based testers.
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Register transaction nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    RegWrite(Id, BigUint, Option<BigUint>, Option<String>), // reg_id, data, overlay_enable, overlay_str
    RegVerify(
        Id,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // reg_id, data, verify_enable, capture_enable, overlay_enable, overlay_str
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// JTAG nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    JTAGWriteIR(u32, BigUint, Option<BigUint>, Option<String>), // size, data, overlay_enable, overlay_str
    JTAGVerifyIR(
        u32,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // size, data, verify_enable, capture_enable, overlay_enable, overlay_str
    JTAGWriteDR(u32, BigUint, Option<BigUint>, Option<String>), // size, data, overlay_enable, overlay_str
    JTAGVerifyDR(
        u32,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // size, data, verify_enable, capture_enable, overlay_enable, overlay_str
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// SWD nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    SWDWriteAP(BigUint, u32, Acknowledgements, Option<BigUint>, Option<String>), // data, A, acknowledgement, overlay_enable, overlay_str
    // SWDBufferedWriteAP(),
    SWDVerifyAP(
        BigUint,
        u32,
        Acknowledgements,
        Option<bool>,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>
    ), // data, A, acknowledgement, parity_compare, verify_enable, capture_enable, overlay_enable, overlay_str
    SWDWriteDP(BigUint, u32, Acknowledgements, Option<BigUint>, Option<String>), // data, A, acknowledgement, overlay_enable, overlay_str
    // SWDBufferedWriteDP(),
    SWDVerifyDP(
        BigUint,
        u32,
        Acknowledgements,
        Option<bool>,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>
    ), // data, A, acknowledgement, parity_compare, verify_enable, capture_enable, overlay_enable, overlay_str
    SWDLineReset,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Arm Debug nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    // ArmDebugMemAPWrite(MemAP, BigUint, BigUint), // mem_ap_id, addr, data
    // ArmDebugMemAPRead(MemAP, BigUint, BigUint), // mem_ap_id, addr, data
    // ArmDebugMemAPCapture(MemAP, BigUint), // mem_ap_id, addr
    ArmDebugMemAPWriteReg(MemAP), // mem_ap_id - 
    ArmDebugMemAPWriteInternalReg(MemAP), // mem_ap_id - 
    ArmDebugMemAPVerifyReg(MemAP),
    ArmDebugMemAPVerifyInternalReg(MemAP),
    // ArmDebugMemAPRead(MemAP, BigUint, BigUint), // mem_ap_id, addr, data
    // ArmDebugMemAPCapture(MemAP, BigUint), // mem_ap_id, addr
    // ArmDebugMemAPVerifyIDR(MemAP), // mem_ap_id - Verifies the IDR for the given MemAP
    ArmDebugWriteDP(ArmDebug), // arm_debug_id - Generic write of a DP register
    ArmDebugVerifyDP(ArmDebug), // arm_debug_id - Generic verify of a DP register
    // ArmDebugVerifyDPIDR(usize), // arm_debug_id - Verify the DP's IDR
    // ArmDebugPowerUp(usize), // arm_debug_id - Write power up bits in DP ctrl/stat
    // ArmDebugVerifyPowerUp(usize), // arm_debug_id - Verify the DP ctrl/stat power up bits are set
    ArmDebugSwjJTAGToSWD(ArmDebug), // arm_debug_id - Switch DP from JTAG to SWD
    ArmDebugSwjSWDToJTAG(ArmDebug), // arm_debug_id - Switch DP from SWD to JTAG
    // ArmDebugSWJ__EnterDormant, // Switch DP to dormant
    // ArmDebugSWJ__ExitDormant, // Switch DP from dormant back to whatever it was prior to entering dormant.

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
    Flow(String),

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
