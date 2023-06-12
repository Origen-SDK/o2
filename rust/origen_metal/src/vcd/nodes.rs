use crate::vcd;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum VCD {
    Root,
    Integer(i64),
    Float(f64),
    String(String),
    Unknown,
    HeaderSection,
    Comment(String),
    Date(String),
    Version(String), // version info is in text format
    Scope(vcd::ScopeType,String),
    TimeScale(u32,vcd::TimeUnit),
    Var(vcd::VarType,u32,String,String,Option<String>),  // type, size, identifier_code, reference (signal name), scope
    UpScope,
    EndDefinitions,
    VcdClose,
    DataSection,
    DumpAll,
    DumpOff,
    DumpOn,
    DumpVars,
    DumpPortsAll,
    DumpPortsOff,
    DumpPortsOn,
    DumpPorts,
    SimulationTime(u32),
    ValueChange(vcd::ValueChangeType,String,String)      // type, value, identifier_code
}

impl std::fmt::Display for VCD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            _ => write!(f, "{}", format!("{:?}", self)),
        }
    }
}