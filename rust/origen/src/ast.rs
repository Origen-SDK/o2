use num_bigint::BigUint;
use std::fmt;

type Id = usize; 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Deleted,
    Test(String),  // The top-level node type
    Comment(String),
    PinWrite(Id, u64),
    PinVerify(Id, u64),
    RegWrite(Id, BigUint),
    RegVerify(Id, BigUint),
    Cycle(u64),
}

pub enum Return {
    /// This is the value returned by the default Processor trait handlers
    /// and is used to indicated that a given processor has not implemented a
    /// handler for a given node type
    Unimplemented,
    /// The node should be deleted from the output AST
    Delete,
    ///
    ProcessChildren,
    Unmodified,
    Replace(Node),
    Inline(Vec<Box<Node>>)
}

#[derive(Clone, Debug)]
pub struct Node {
    kind: NodeKind,
    //filename: Option<String>,
    //lineno: Option<usize>,
    children: Vec<Box<Node>>
}

// Implements default handlers for all node types
trait Processor {
    // This will be called for all nodes unless a dedicated handler
    // handler exists for the given node type
    fn on_all(&mut self, _node: &Node) -> Return {
        Return::ProcessChildren
    }

    fn on_test(&mut self, _name: &str, _node: &Node) -> Return {
        Return::Unimplemented
    }

    fn on_comment(&mut self, _msg: &str, _node: &Node) -> Return {
        Return::Unimplemented
    }

    fn on_cycle(&mut self, _repeat: u64, _node: &Node) -> Return {
        Return::Unimplemented
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = ToString::new();
        write!(f, "{}", p.run(self))
    }
}

impl Node {
    fn deleted() -> Node {
        Node{kind: NodeKind::Deleted, children: Vec::new()}
    }

    fn process(&self, processor: &mut dyn Processor) -> Option<Node> {
        // Call the dedicated handler for this node if it exists
        let r = match &self.kind {
            NodeKind::Test(name) => processor.on_test(&name, &self),
            NodeKind::Comment(msg) => processor.on_comment(&msg, &self),
            NodeKind::Cycle(repeat) => processor.on_cycle(*repeat, &self),
            _ => Return::Unimplemented,
        };
        // If not, call the default handler all nodes handler
        let r = match r {
            Return::Unimplemented => processor.on_all(&self),
            _ => r,
        };
        // Now decide what action to take and what to return based on the return
        // code from the node's handler.
        match r {
            Return::Delete => None,
            Return::ProcessChildren =>  {
                for child in &self.children {
                    child.process(processor);
                };
                Some(self.clone())
            },
            Return::Unmodified => Some(self.clone()),
            Return::Replace(node) => Some(node),
            Return::Inline(nodes) => Some(self.clone()),
            _ => None,
        }
    }

    fn process_children(&self, processor: &mut dyn Processor) -> Vec<Box<Node>> {
        self.children.iter().map(|node|
            match node.process(processor) {
                Some(n) => Box::new(n),
                None => Box::new(Node::deleted()),
            }
        ).collect()
    }
}

// This is an example of what a compiler stage will look like, only implements
// handlers for the node types it cares about.
// The others will have their children processed by default, but will otherwise
// be ignored.
// This one would print out every comment in the AST.
struct CommentPrinter {}

impl Processor for CommentPrinter {
    fn on_comment(&mut self, msg: &str, node: &Node) -> Return {
        println!("[COMMENT] - {}", msg);
        Return::Delete
    }
}

// This is an example of what a compiler stage will look like, only implements
// handlers for the node types it cares about.
// The others will have their children processed by default, but will otherwise
// be ignored.
// This one would print out every comment in the AST.
struct ToString {
    indent: usize,
    output: String,
}

impl ToString {
    fn new() -> ToString {
        ToString{indent: 0, output: "".to_string()}
    }
    
    fn run(&mut self, node: &Node) -> &str {
        node.process(self);
        &self.output
    }
}

impl Processor for ToString {
    fn on_all(&mut self, node: &Node) -> Return {
        self.output += &" ".repeat(self.indent);
        self.output += &format!("{:?}\n", node.kind);
        self.indent += 4;
        node.process_children(self);
        self.indent -= 4;
        Return::Unmodified
    }
}

//#[derive(Debug, Eq, PartialEq)]
//pub enum AstNode {
//    Pin(PinAction),
//    Register(RegisterAction),
//
//    // Below this line are just test types for now
//    Timeset(String),
//    Cycle {
//        repeat: u32,
//    },
//    Comment(String),
//    Instrument {
//        name: String,
//        data: String,
//        operation: Operation,
//    },
//}



#[cfg(test)]
mod tests {
    use crate::ast::*;

    #[test]
    fn nodes_can_be_created_and_nested() {
        let mut test = Node{kind: NodeKind::Test("trim_vbgap".to_string()), children: Vec::new()};
        let c1 = Node{kind: NodeKind::Comment("Hello".to_string()), children: Vec::new()};
        test.children.push(Box::new(c1));

        let cyc = Node{kind: NodeKind::Cycle(1), children: Vec::new()};
        for i in 0..10 {
            test.children.push(Box::new(cyc.clone()));
        }
        
        let c2 = Node{kind: NodeKind::Comment("Hello Again!".to_string()), children: Vec::new()};
        test.children.push(Box::new(c2));

        test.process(&mut CommentPrinter{});
        
        println!("{}", test);
        
        //let reg_write = 
        //let mut reg_write = 
        assert_eq!(1, 1);
    }
}