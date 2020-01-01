use id_arena::Arena;
use super::ast_node::{AstNode, AstNodeId};

#[derive(Debug)]
pub struct Collector {
    pub collection: Arena::<AstNode>,
}

impl Collector {
    pub fn new() -> Collector {
        Collector { collection: Arena::<AstNode>::new() }
    }
    
    pub fn add_node(&mut self, node: AstNode) -> AstNodeId {
        self.collection.alloc(node)
    }
    
}

#[derive(Debug)]
pub struct CollectionStack {
    stack: Vec<Collector>,
}

impl CollectionStack {
    pub fn new() -> CollectionStack {
        let mut cs = CollectionStack { stack: Vec::new() };
        // ensure there is 1 element on the stack so that it's usable
        cs.new_collection();
        cs
    }
    
    // create a new collection and push it on the end of the stack
    pub fn new_collection(&mut self) {
        self.stack.push(Collector::new());
    }
    
    // pop a collection off the top of the stack and return it's Arena
    // if there is only 1 element in the stack (this is the main collection)
    // an empty Arena is returned
    pub fn pop_collection(&mut self) -> Arena::<AstNode> {
        if self.stack.len() == 1 {
            Arena::<AstNode>::new()
        } else {
            match self.stack.pop() {
                Some(col) => col.collection,
                None => Arena::<AstNode>::new(),
            }
        }
    }
    
    // allocate a node in the Arena at the top of the stack, returning the ID
    pub fn add_node(&mut self, node: AstNode) -> AstNodeId {
        let index = self.stack.len() -1;
        self.stack[index].add_node(node)
    }
}
