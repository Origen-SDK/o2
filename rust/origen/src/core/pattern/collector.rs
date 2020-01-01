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
    stack: Vec<Collector>,
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
