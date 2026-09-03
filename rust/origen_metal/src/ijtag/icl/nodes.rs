#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PortType {
    ScanIn,
    ScanOut,
    ShiftEn,
    CaptureEn,
    UpdateEn,
    DataIn,
    DataOut,
    ToShiftEn,
    ToUpdateEn,
    ToCaptureEn,
    Select,
    ToSelect,
    Reset,
    ToReset,
    Tms,
    ToTms,
    Tck,
    ToTck,
    Clock,
    ToClock,
    Trst,
    ToTrst,
    ToIrSelect,
    Address,
    WriteEn,
    ReadEn,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SignalType {
    Reset,
    Scan,
    Data,
    Clock,
    Tck,
    Tms,
    Trst,
    ShiftEn,
    CaptureEn,
    UpdateEn,
    HierarchicalData,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MuxType {
    Scan,
    Data,
    Clock,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccessLinkStandard {
    Std1149_1_2001,
    Std1149_1_2013,
}

/// Attributes used by the syntax-preserving IEEE 1687-2014 ICL AST.
///
/// Declaration names and values are represented by child nodes. This keeps scalar, vector, and
/// hierarchical identifiers uniform in every context and preserves expression structure.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ICL {
    Root,
    SourceFile(String),
    Comment(String),

    NameSpace,
    UseNameSpace,
    Module,
    Port(PortType),
    Instance,
    ModuleReference,
    ScanRegister,
    DataRegister,
    LogicSignal,
    Mux(MuxType),
    MuxSelection(MuxType),
    OneHotScanGroup,
    OneHotDataGroup,
    ScanInterface,
    ScanInterfaceChain,
    AccessLink(AccessLinkStandard),
    GenericAccessLink,
    GenericAccessLinkBody(String),
    BsdlInstruction,
    Alias,
    Enumeration,
    EnumerationItem,
    Parameter,
    LocalParameter,
    Attribute,

    Source,
    Enable,
    RefEnum,
    DefaultLoadValue,
    ActivePolarity(bool),
    DifferentialInvOf,
    FrequencyMultiplier,
    FrequencyDivider,
    Period,
    InputPortConnection,
    AllowBroadcastOnScanInterface,
    AddressValue,
    ScanInSource,
    CaptureSource,
    ResetValue,
    WriteEnSource,
    WriteDataSource,
    ReadCallBack,
    ReadDataSource,
    WriteCallBack,
    IProcReference,
    IProcArgument(String),
    PortReference,
    ScanInterfaceReference,
    BsdlEntity,
    ScanInterfaces,
    ActiveSignals,
    AccessTogether,
    ApplyEndState,

    Identifier(String),
    VectorIdentifier,
    HierarchicalIdentifier,
    Index,
    Range,
    ParameterReference(String),
    StringLiteral(String),
    Number(String),
    TimeUnit(String),
    Signal(SignalType),
    Concatenation,
    Alternatives,
    Invert,

    IntegerExpression,
    IntegerTerm,
    Parentheses,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    LogicExpression,
    LogicBitwiseExpression,
    LogicEqualityExpression,
    LogicConcatenation,
    BooleanAnd,
    BooleanOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BooleanNot,
    Equal,
    NotEqual,
}

impl std::fmt::Display for ICL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
