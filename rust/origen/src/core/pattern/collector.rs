use super::ast_node::{AstNode, AstNodeId};
use crate::error::Error;
use crate::Result;
use std::fs::File;
use std::io::prelude::*;

// The collector is a vec of sequential node id's

#[derive(Debug, Eq, PartialEq)]
pub struct Collector {
    pub collection: Vec<AstNodeId>,
}

impl Collector {
    pub fn new() -> Collector {
        Collector {
            collection: Vec::new(),
        }
    }
}

// The collection stack is a Vector of Vectors used for 2 purposes:
//   1) Index 0 is the main collection of sequential ast nodes id's to be processed
//   2) For node types that will have children a new stack index is created
//      once the children are collected, that stack index is moved to the parent node

#[derive(Debug)]
pub struct CollectionStack {
    // pub for now, will add a getter or iterator
    pub stack: Vec<Collector>,
}

impl CollectionStack {
    // Create a new CollectionStack ensuring that the main collection is created
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

    // return the main collection of node id's
    pub fn main_collection(&self) -> &Collector {
        &self.stack[0]
    }

    // allocate a node in the collection at the top of the stack
    pub fn add_node(&mut self, node: &AstNodeId) {
        let index = self.stack.len() - 1;
        self.stack[index].collection.push(*node)
    }

    // clear all contents of the stack
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

// NodeCollection holds the actual Ast Nodes and a CollectionStack (references to the nodes in the order they are to be processed)
pub struct NodeCollection {
    pub nodes: Vec<AstNode>,
    pub stack: CollectionStack,
}

impl NodeCollection {
    pub fn new() -> NodeCollection {
        NodeCollection {
            nodes: Vec::new(),
            stack: CollectionStack::new(),
        }
    }

    // Stores the AstNode and adds it's ID to the active collection of the stack
    // Returns the node id (which is it's index in the node vec)
    pub fn add_node(&mut self, node: AstNode) -> AstNodeId {
        self.nodes.push(node);
        self.stack.add_node(&(self.nodes.len() - 1));
        self.nodes.len() - 1
    }

    pub fn get_mut_node(&mut self, id: AstNodeId) -> Result<&mut AstNode> {
        match self.nodes.get_mut(id) {
            Some(x) => Ok(x),
            None => return Err(Error::new(&format!("Node does not exist: {}", id))),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.stack.clear();
    }

    // return the main collection of node id's
    pub fn main_collection(&self) -> &Collector {
        self.stack.main_collection()
    }
    
    // coming soon - iterate through the stack and output text representation of the AST
    pub fn to_text(&self, file_name: &str) {
        let mut file = match File::create(file_name) {
            Err(_e) => panic!("could not create {}", file_name),
            Ok(f) => f,
        };
        self.all_nodes_to_file(&mut file, self.main_collection(), "s");
    }
    
    pub fn all_nodes_to_file(&self, file: &mut File, nodes: &Collector, prefix: &str){
        for node_id in nodes.collection.iter() {
            if let Some(node) = self.nodes.get(*node_id) {
                match node {
                    AstNode::Pin(n) => {file.write(format!("{}({})", prefix, n.to_string()).as_bytes());},
                    AstNode::Register(n) => {
                        file.write(format!("{}({})", prefix, n.to_string()).as_bytes());
                        self.all_nodes_to_file(file, &n.children, &("  ".to_owned() + prefix));
                       },
                    _ => {file.write(format!("unprocessed node").as_bytes());},
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ast_node::AstNode;
    use super::super::operation::Operation;
    use super::super::pin_action::PinAction;
    use super::super::register_action::RegisterAction;
    use super::*;

    #[test]
    fn stack_works_to_collect_children() {
        // this holds all nodes
        let mut pattern_nodes = NodeCollection::new();
        // the stack holds vectors of node ID's for sequential processing

        // place a few nodes in the stack
        pattern_nodes.add_node(AstNode::Timeset("tp0".to_string()));
        pattern_nodes.add_node(AstNode::Pin(PinAction::new("pa0", "0", Operation::Write)));

        // now create a node with children
        let mut reg_action = RegisterAction::new("ctrl", &0x300, "0xffee0011", Operation::Read);
        // new collection for collecting child nodes
        pattern_nodes.stack.new_collection();
        // now all actions should go to the new collection
        pattern_nodes.add_node(AstNode::Pin(PinAction::new("pa0", "1", Operation::Write)));
        // done with register read, now pop children into the register node
        reg_action.children.collection = pattern_nodes.stack.pop_collection();
        // place the now completed register node into the collection
        let ra_item = pattern_nodes.add_node(AstNode::Register(reg_action));

        // check sizes, ugly code
        assert_eq!(pattern_nodes.stack.stack.len(), 1);
        assert_eq!(pattern_nodes.stack.stack[0].collection.len(), 3);
        // get the register action node back and check the length of the children
        if let Some(reg_ast_node) = pattern_nodes.nodes.get(ra_item) {
            match reg_ast_node {
                AstNode::Register(reg_action) => {
                    assert_eq!(reg_action.children.collection.len(), 1)
                }
                _ => panic!("didn't get a register action back"),
            }
        }

        // iterate through the nodes to generate the final pattern output
        process_all_nodes(&pattern_nodes.main_collection(), &pattern_nodes);
    }

    // simple processor example - this will be deleted in the future
    fn process_all_nodes(nodes: &Collector, pattern: &NodeCollection) {
        for node_id in nodes.collection.iter() {
            if let Some(node) = pattern.nodes.get(*node_id) {
                match node {
                    AstNode::Pin(pa) => println!("Call the method that updates the output pattern vector string, sending pa(PinAction struct)"),
                    AstNode::Register(ra) => {
                        println!("for an ascii pattern, print a comment with the register info, then process all children");
                        process_all_nodes(&ra.children, pattern);
                    },
                    AstNode::Comment(comment) => println!("print the comment to the output"),
                    _ => println!("other stuff"),
                }
            }
        }
    }
}
