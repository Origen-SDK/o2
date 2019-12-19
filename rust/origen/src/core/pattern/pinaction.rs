//! Defines the set of actions associated with a pattern pin action
pub use super::operation::Operation;

pub struct PinAction {
    name: String,
    data: String,
    operation: Operation,
}

impl PinAction {
    // This exists to add window dressing to the data string. Default expected will be.
    // TODO: "0x" will be added if no format designator is present.
    pub fn new(name: &str, data: &str, operation: Operation) -> PinAction {
        PinAction {
            name: name.to_string(),
            data: data.to_string(),
            operation: operation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinaction_readable() {
        let pa = PinAction { name: "porta_01".to_string(), data: "0".to_string(), operation: Operation::Read, };
        assert_eq!(pa.name, "porta_01");
        assert_eq!(pa.data, "0");
        assert_eq!(pa.operation.to_string(), "read");
    }
    
    #[test]
    fn new_creates_struct() {
        let pa = PinAction::new("pingroup_name", "0x55", Operation::DriveMem);
        assert_eq!(pa.name, "pingroup_name");
        assert_eq!(pa.data, "0x55");
        assert_eq!(pa.operation.to_string(), Operation::DriveMem.to_string());
    }
}
