use super::stil;
use num_bigint::BigUint;
type Id = usize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Attrs {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Data Types
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Integer(u64),
    SignedInteger(i64),
    Float(f64),
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Test (pat gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Test(String),
    Comment(u8, String), // level, msg
    PinWrite(Id, u128),
    PinVerify(Id, u128),
    RegWrite(Id, BigUint, Option<BigUint>, Option<String>), // reg_id, data, overlay_enable, overlay_str
    RegVerify(
        Id,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // reg_id, data, verify_enable, capture_enable, overlay_enable, overlay_str
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
    Cycle(u32, bool), // repeat (0 not allowed), compressable

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
    STILName(String),
    STILSIUnit(String),
    STILEngPrefix(String),
    STILAdd,
    STILSubtract,
    STILMultiply,
    STILDivide,
    STILParens,
    STILNumberWithUnit,
    STILNumber,
    STILPoint,
    STILExp,
    STILMinus,
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
