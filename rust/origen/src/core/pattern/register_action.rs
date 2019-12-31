//! Defines the set of actions associated with a register action
pub use super::operation::Operation;
use id_arena::Arena;
pub use super::ast_node::AstNode;

#[derive(Debug, Eq, PartialEq)]
pub struct RegisterAction {
    pub name: String,
    pub address: u64,
    pub data: String,
    pub operation: Operation,
    // register action can contain arbitrary number of child actions
    pub children: Arena::<AstNode>,
}

impl RegisterAction {
    // This exists to add window dressing to the data string. Default expected will be hex.
    // TODO: "0x" will be added if no format designator is present.
    pub fn new(name: &str, address: &u64, data: &str, operation: Operation) -> RegisterAction {
        RegisterAction {
            name: name.to_string(),
            address: *address,
            data: data.to_string(),
            operation: operation,
            children: Arena::<AstNode>::new(),
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("register: {}, address: {}, data: {}, operation: {}, num children: {}", self.name, self.address, self.data, self.operation.to_string(), self.children.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn converts_to_string(){
        let ra_node = RegisterAction::new("cntrl", &300, "0x40", Operation::Read);
        assert_eq!(ra_node.to_string(), "register: cntrl, address: 300, data: 0x40, operation: read, num children: 0");
    }
    
    #[test]
    fn instantiates_new_mutable_arena() {
        let mut ra_node = RegisterAction::new("cntrl", &300, "0x40", Operation::Read);
        ra_node.children.alloc(AstNode::Timeset("tp0".to_string()));
        assert_eq!(ra_node.children.len(), 1);
    }
}