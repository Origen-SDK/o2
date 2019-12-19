//! Defines the set of actions associated with pattern generation
// TODO: Pick a better type for data. Integers will not suffice for very large registers or banks of pins
// Enum Structs will be defined for most action types to hold their associated info
//
// I envision the internal pattern storage type being a vector of type Pattern::Action
// Code processing the Pattern will use match with arms to process the type of Actions that are supported for the given output

// TODO: The operation/action type enum should come from the module that models the object (pins, regs, protocol, etc.)
use super::operation::Operation;
use super::pinaction::PinAction;

pub enum Action {
    Pin(PinAction),
    Timeset{name: String},
    Cycle{repeat: u32},
    // likely need a larger storage type for address and data, or maybe generics to provide options
    // These are place holders of action types as I think of them
    // Register{name: String, address: u32, data: u32, operation: Operation, start_stop: Operation},
    // Driver{name: String, operation: Operation, data: u32, size: u32, target: String, start_stop: Operation},
    // Comment(String),
    // Instrument{name: String, data: String, operation: Operation},
}
