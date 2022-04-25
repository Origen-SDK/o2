use crate::stil;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum STIL {
    Root,
    Integer(i64),
    Float(f64),
    String(String),
    Unknown,
    Version(u32, u32), // major, minor
    Header,
    Title(String),
    Date(String),
    Source(String),
    History,
    Annotation(String),
    Include(String, Option<String>),
    Signals,
    Signal(String, stil::SignalType), // name, type
    Termination(stil::Termination),
    DefaultState(stil::State),
    Base(stil::Base, String),
    Alignment(stil::Alignment),
    ScanIn(u32),
    ScanOut(u32),
    DataBitCount(u32),
    SignalGroups(Option<String>),
    SignalGroup(String),
    SigRefExpr,
    TimeExpr,
    SIUnit(String),
    EngPrefix(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    Parens,
    NumberWithUnit,
    PatternExec(Option<String>),
    CategoryRef(String),
    SelectorRef(String),
    TimingRef(String),
    PatternBurstRef(String),
    PatternBurst(String),
    SignalGroupsRef(String),
    MacroDefs(String),
    Procedures(String),
    ScanStructuresRef(String),
    Start(String),
    Stop(String),
    Terminations,
    TerminationItem,
    PatList,
    Pat(String),
    Label(String),
    Timing(Option<String>),
    WaveformTable(String),
    Period,
    Inherit(String),
    SubWaveforms,
    SubWaveform,
    Waveforms,
    Waveform,
    WFChar(String),
    Event,
    EventList(Vec<char>),
    Spec(Option<String>),
    Category(String),
    SpecItem,
    TypicalVar(String),
    SpecVar(String),
    SpecVarItem(stil::Selector),
    Variable(String),
    Selector(String),
    SelectorItem(String, stil::Selector),
    ScanStructures(Option<String>),
    ScanChain(String),
    ScanLength(u64),
    ScanOutLength(u64),
    ScanCells,
    ScanMasterClock,
    ScanSlaveClock,
    ScanInversion(u8),
    ScanInName(String),
    ScanOutName(String),
    Not,
    Pattern(String),
    TimeUnit,
    Vector,
    CyclizedData,
    NonCyclizedData,
    Repeat(u64),
    WaveformFormat,
    HexFormat(Option<String>),
    DecFormat(Option<String>),
    Data(String),
    TimeValue(u64),
    WaveformRef(String),
    Condition,
    Call(String),
    Macro(String),
    Loop(u64),
    MatchLoop(Option<u64>),
    Goto(String),
    BreakPoint,
    IDDQ,
    StopStatement,
}

impl std::fmt::Display for STIL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            _ => write!(f, "{}", format!("{:?}", self)),
        }
    }
}
