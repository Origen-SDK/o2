//! Defines the set of actions associated with a pattern pin action
pub use super::operation::Operation;

pub struct PinAction {
    name: String,
    data: String,
    operation: Operation,
}

#[cfg(test)]
mod tests {
    use crate::core::pattern::pinaction;

    #[test]
    fn pinaction_readable() {
        let pa = pinaction::PinAction { name: "porta_01".to_string(), data: "0".to_string(), operation: pinaction::Operation::Read, };
        assert_eq!(pa.name, "porta_01");
        assert_eq!(pa.data, "0");
        assert_eq!(pa.operation.to_string(), "read");
    }
}
