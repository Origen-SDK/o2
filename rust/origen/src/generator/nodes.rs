use super::utility::transaction::Transaction;
use crate::services::swd::Acknowledgements;
use crate::testers::SupportedTester;
use indexmap::IndexMap;
use crate::om::TypedValueMap;

pub type Id = usize;
type Metadata = Option<TypedValueMap>;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum PAT {
    /// A simple node that is quick to write and use in processor unit tests
    T(usize),
    /// This will be ignored by all processors, so can be used to indicate the absence of a node
    Nil,

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
    PinGroupAction(usize, Vec<String>, Metadata),
    PinAction(usize, String, Metadata),
    Capture(crate::Capture, Metadata),
    EndCapture(Option<usize>),
    Overlay(crate::Overlay, Metadata),
    EndOverlay(Option<String>, Option<usize>), // Label, PinID
    Opcode(String, IndexMap<String, String>),  // Opcode, Arguments<Argument Key, Argument Value>
    Cycle(u32, bool),                          // repeat (0 not allowed), compressable
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
}

impl std::fmt::Display for PAT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PAT::PinGroupAction(grp_id, _actions, _metadata) => {
                let dut = crate::dut();
                write!(
                    f,
                    "{}",
                    format!("{:?} -> ({})", self, &dut.pin_groups[*grp_id].name)
                )
            }
            PAT::PinAction(id, _actions, _metadata) => {
                let dut = crate::dut();
                write!(f, "{}", format!("{:?} -> ({})", self, &dut.pins[*id].name))
            }
            _ => write!(f, "{}", format!("{:?}", self)),
        }
    }
}
