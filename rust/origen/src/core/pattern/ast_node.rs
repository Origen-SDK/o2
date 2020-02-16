use super::conditionals::*;
use super::operation::Operation;
use super::pin_action::PinAction;
use super::register_action::RegisterAction;

pub type AstNodeId = usize;

#[derive(Debug, Eq, PartialEq)]
pub enum AstNode {
    Pin(PinAction),
    Register(RegisterAction),

    // Below this line are just test types for now
    Timeset(String),
    Cycle {
        repeat: u32,
    },
    Comment(String),
    Instrument {
        name: String,
        data: String,
        operation: Operation,
    },
    // Driver{name: String, operation: Operation, data: u32, size: u32, target: String, start_stop: Operation},
    CondIfTrue(ConditionalIf),
    CondElse(ConditionalElse),
}
