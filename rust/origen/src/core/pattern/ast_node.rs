use id_arena::{Arena, Id};
use super::operation::Operation;
use super::pinaction::PinAction;
use super::register_action::RegisterAction;

pub type AstNodeId = Id<AstNode>;

#[derive(Debug, Eq, PartialEq)]
pub enum AstNode {
    Pin(PinAction),
    
    // Below this line are just test types for now
    Timeset(String),
    Cycle{repeat: u32},
    Comment(String),
    Instrument{name: String, data: String, operation: Operation},
    Register(RegisterAction),
    // Driver{name: String, operation: Operation, data: u32, size: u32, target: String, start_stop: Operation},
}