//! Defines the set of actions associated with pattern generation
// TODO: Pick a better type for data integers will not suffice for very large registers or banks of pins

use super::operation::Operation;
use super::pinaction::PinAction;

pub enum Action {
    Pin(PinAction),
    Timeset{name: String},
    Cycle{repeat: u32},
    // likely need a larger storage type for address and data
    Register{name: String, address: u32, data: u32, operation: Operation, start_stop: Operation},
    Driver{name: String, operation: Operation, data: u32, size: u32, target: String, start_stop: Operation},
}
