//! The collector stack is a Vector of Vectors used for 2 purposes:
//!   1) Index 0 is the main collection of sequential ast nodes to be processed
//!   2) For node types that will have children a new stack index is created
//!      once the children are collected, that stack index is moved to the parent node

use super::ast_node::AstNodeId;

#[derive(Debug)]
pub struct Collector {
    pub collection: Vec<AstNodeId>,
}

impl Collector {
    pub fn new() -> Collector {
        Collector { collection: Vec::new() }
    }    
}

#[derive(Debug)]
pub struct CollectionStack {
    // pub for now, will add a getter or iterator
    pub stack: Vec<Collector>,
}

impl CollectionStack {
    pub fn new() -> CollectionStack {
        let mut cs = CollectionStack { stack: Vec::new() };
        // ensure there is 1 element on the stack the main collection
        cs.new_collection();
        cs
    }
    
    // create a new collection and push it on the end of the stack
    pub fn new_collection(&mut self) {
        self.stack.push(Collector::new());
    }
    
    // pop a collection off the top of the stack and return it's Vec
    // if there is only 1 element in the stack (this is the main collection)
    // an empty Vec is returned
    pub fn pop_collection(&mut self) -> Vec<AstNodeId> {
        if self.stack.len() == 1 {
            Vec::new()
        } else {
            match self.stack.pop() {
                Some(col) => col.collection,
                None => Vec::new(),
            }
        }
    }
    
    // allocate a node in the collection at the top of the stack
    pub fn add_node(&mut self, node: &AstNodeId) {
        let index = self.stack.len() -1;
        self.stack[index].collection.push(*node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ast_node::{AstNode, AstNodeId};
    use super::super::register_action::RegisterAction;
    use super::super::pin_action::PinAction;
    use super::super::operation::Operation;
    use id_arena::Arena;
    
    #[test]
    fn stack_works_to_collect_children(){
        // this holds all nodes
        let mut pattern_nodes = Arena::<AstNode>::new();
        // the stack holds vectors of node ID's for sequential processing
        let mut collection_stack = CollectionStack::new();
        
        // place a few nodes in the stack
        collection_stack.add_node(&pattern_nodes.alloc(AstNode::Timeset("tp0".to_string())));
        collection_stack.add_node(&pattern_nodes.alloc(AstNode::Pin(PinAction::new("pa0", "0", Operation::Write))));
        
        // now create a node with children
        let mut reg_action = RegisterAction::new("ctrl", &0x300, "0xffee0011", Operation::Read);
        // new collection for collecting child nodes
        collection_stack.new_collection();
        // now all actions should go to the new collection
        collection_stack.add_node(&pattern_nodes.alloc(AstNode::Pin(PinAction::new("pa0", "1", Operation::Write))));
        // done with register read, now pop children into the register node
        reg_action.children = collection_stack.pop_collection();
        // place the now completed register node into the collection
        collection_stack.add_node(&pattern_nodes.alloc(AstNode::Register(reg_action)));
        
        // check sizes
        assert_eq!(collection_stack.stack.len(), 1);
        assert_eq!(collection_stack.stack[0].collection.len(), 3);
        // get the register action node back and check the length of the children
        // ugly code here, this isn't how you'd normally do this
        let reg_ast_node = &pattern_nodes[collection_stack.stack[0].collection[2]];
        match reg_ast_node {
            AstNode::Register(reg_action) => assert_eq!(reg_action.children.len(), 1),
            _ => panic!("didn't get a register action back"),
        }
    }
}