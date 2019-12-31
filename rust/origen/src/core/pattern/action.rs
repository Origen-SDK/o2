//! This file is now obsolete and will be deleted
//! Defines the set of actions associated with pattern generation
// String is used for data to allow very large (ex. 300-bit) numbers
// Enum Structs will be defined for most action types to hold their associated info
//

use super::operation::Operation;
use super::pinaction::PinAction;

pub enum Action {
    Pin(PinAction),
    // Below this line are just test types for now
    Timeset(String),
    Cycle{repeat: u32},
    // These are place holders of action types as I think of them
    // Register{name: String, address: u32, data: u32, operation: Operation, start_stop: Operation},
    // Driver{name: String, operation: Operation, data: u32, size: u32, target: String, start_stop: Operation},
    // Comment(String),
    // Instrument{name: String, data: String, operation: Operation},
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Pin(p) => format!("PinAction -> {}", p.to_string()),
            Action::Timeset(s) => format!("Timeset -> {}", s.to_string()),
            Action::Cycle{repeat: r} => format!("Cycle -> repeat: {}", r.to_string()),
        }
    }
}