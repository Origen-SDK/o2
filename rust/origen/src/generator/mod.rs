mod processors;

use num_bigint::BigUint;
use processors::ToString;
use std::fmt;

type Id = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nodes {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,  
    /// The top-level node type
    Test(String),
    Comment(String),
    PinWrite(Id, u64),
    PinVerify(Id, u64),
    RegWrite(Id, BigUint),
    RegVerify(Id, BigUint),
    Cycle(u64),
}

#[derive(Clone, Debug)]
pub struct Node {
    attrs: Nodes,
    meta: Option<Meta>,
    children: Vec<Box<Node>>,
}

#[derive(Clone, Debug)]
pub struct Meta {
    filename: Option<String>,
    lineno: Option<usize>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = ToString::new();
        write!(f, "{}", p.run(self))
    }
}

impl Node {
    fn inline(nodes: Vec<Box<Node>>) -> Node {
        Node {
            attrs: Nodes::_Inline,
            meta: None,
            children: nodes,
        }
    }

    /// Returns a new node which is the output of the node processed by the
    /// given processor.
    /// Returning None means that the processor has decided that the node should
    /// be deleted from the next stage AST.
    fn process(&self, processor: &mut dyn Processor) -> Option<Node> {
        // Call the dedicated handler for this node if it exists
        let r = match &self.attrs {
            Nodes::Test(name) => processor.on_test(&name, &self),
            Nodes::Comment(msg) => processor.on_comment(&msg, &self),
            Nodes::Cycle(repeat) => processor.on_cycle(*repeat, &self),
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
            Return::ProcessChildren => {
                let nodes = self.process_children(processor);
                Some(self.replace_children(nodes))
            }
            Return::Unmodified => Some(self.clone()),
            Return::Replace(node) => Some(node),
            // We can't return multiple nodes from this function, so we return them
            // wrapped in a meta-node and the process_children method will identify
            // this and remove the wrapper to inline the contained nodes.
            Return::Inline(nodes) => Some(Node::inline(nodes)),
            _ => None,
        }
    }

    /// Returns a new vector containing processed versions of the node's children
    fn process_children(&self, processor: &mut dyn Processor) -> Vec<Box<Node>> {
        let mut output: Vec<Box<Node>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor) {
                if let Nodes::_Inline = node.attrs {
                    for c in node.children {
                        output.push(c);
                    }
                } else {
                    output.push(Box::new(node));
                }
            }
        };
        output
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    fn replace_children(&self, nodes: Vec<Box<Node>>) -> Node {
        let new_node = Node {
            attrs: self.attrs.clone(),
            meta: self.meta.clone(),
            children: nodes,
        };
        new_node
    }

    /// Returns a new node which is a copy of self with its attrs replaced
    /// by the given attrs.
    fn replace_attrs(&self, attrs: Nodes) -> Node {
        let new_node = Node {
            attrs: attrs,
            meta: self.meta.clone(),
            children: self.children.clone(),
        };
        new_node
    }
}

pub enum Return {
    /// This is the value returned by the default Processor trait handlers
    /// and is used to indicated that a given processor has not implemented a
    /// handler for a given node type. Implementations of the Processor trait
    /// should never return this type.
    Unimplemented,
    /// Deleted the node from the output AST.
    Delete,
    /// Process the node's children, replacing it's current children with their
    /// processed counterparts in the output AST.
    ProcessChildren,
    /// Clones the node (and all of its children) into the output AST.
    Unmodified,
    /// Replace the node in the output AST with the given node.
    Replace(Node),
    /// Replace the node in the output AST with the given nodes, the vector wrapper
    /// will be removed and the nodes will be placed inline with where the current
    /// node is/was.
    Inline(Vec<Box<Node>>),
}

// Implements default handlers for all node types
trait Processor {
    // This will be called for all nodes unless a dedicated handler
    // handler exists for the given node type. It means that by default, all
    // nodes will have their children processed by all processors.
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

// This is an example of what a compiler stage will look like, only implements
// handlers for the node types it cares about.
// The others will have their children processed by default, but will otherwise
// be ignored.
// This one would print out every comment in the AST.
struct CommentPrinter {}

impl Processor for CommentPrinter {
    fn on_comment(&mut self, msg: &str, _node: &Node) -> Return {
        println!("[COMMENT] - {}", msg);
        Return::Unmodified
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::*;

    #[test]
    fn nodes_can_be_created_and_nested() {
        let mut test = Node {
            attrs: Nodes::Test("trim_vbgap".to_string()),
            meta: None,
            children: Vec::new(),
        };
        let c1 = Node {
            attrs: Nodes::Comment("Hello".to_string()),
            meta: None,
            children: Vec::new(),
        };
        test.children.push(Box::new(c1));

        let cyc = Node {
            attrs: Nodes::Cycle(1),
            meta: None,
            children: Vec::new(),
        };
        for _i in 0..10 {
            test.children.push(Box::new(cyc.clone()));
        }

        let c2 = Node {
            attrs: Nodes::Comment("Hello Again!".to_string()),
            meta: None,
            children: Vec::new(),
        };
        test.children.push(Box::new(c2));

        test.process(&mut CommentPrinter {});

        println!("{}", test);

        //let reg_write =
        //let mut reg_write =
        assert_eq!(1, 1);
    }
}
