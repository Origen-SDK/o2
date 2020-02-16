//! Defines the set of actions associated with a register action
pub use super::ast_node::AstNodeId;
pub use super::collector::Collector;
pub use super::operation::Operation;

#[derive(Debug, Eq, PartialEq)]
pub struct RegisterAction {
    pub name: String,
    pub address: u64,
    pub data: String,
    pub operation: Operation,
    // register action can contain arbitrary number of child actions
    pub children: Collector,
}

impl RegisterAction {
    // TODO: name and address can be replaced by a register ID in the future
    pub fn new(name: &str, address: &u64, data: &str, operation: Operation) -> RegisterAction {
        RegisterAction {
            name: name.to_string(),
            address: *address,
            data: data.to_string(),
            operation: operation,
            children: Collector::new(),
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "register: {}, address: {}, data: {}, operation: {}, num children: {}",
            self.name,
            self.address,
            self.data,
            self.operation.to_string(),
            self.children.collection.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::ast_node::AstNode;
    use super::super::collector::NodeCollection;
    use super::*;

    #[test]
    fn converts_to_string() {
        let ra_node = RegisterAction::new("cntrl", &300, "0x40", Operation::Read);
        assert_eq!(
            ra_node.to_string(),
            "register: cntrl, address: 300, data: 0x40, operation: read, num children: 0"
        );
    }

    #[test]
    fn instantiates_new_mutable_children_vec() {
        let mut ast_nodes = NodeCollection::new();
        let mut ra_node = RegisterAction::new("cntrl", &300, "0x40", Operation::Read);
        ra_node
            .children
            .collection
            .push(ast_nodes.add_node(AstNode::Timeset("tp0".to_string())));
        assert_eq!(ra_node.children.collection.len(), 1);
    }
}
