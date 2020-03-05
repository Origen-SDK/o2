use super::processors::ToString;
use crate::generator::processor::*;
use crate::{Error, Result};
use num_bigint::BigUint;
use std::fmt;

#[derive(Debug)]
pub struct AST {
    nodes: Vec<Node>,
}

impl AST {
    /// Create a new AST with the given node as the top-level
    pub fn new(node: Node) -> AST {
        AST { nodes: vec![node] }
    }

    /// Push a new terminal node into the AST
    pub fn push(&mut self, node: Node) {
        self.nodes.last_mut().unwrap().add_child(node);
    }

    /// Push a new node into the AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// An reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of AST application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len()
    }

    /// Close the currently open node
    pub fn close(&mut self, ref_id: usize) -> Result<()> {
        if self.nodes.len() != ref_id {
            return Err(Error::new(
                "Attempt to close a parent AST node without first closing all its children, it looks like you have either forgotten to close an open node or given the wrong reference ID"
            ));
        }
        if ref_id == 1 {
            return Err(Error::new("The top-level AST node can never be closed"));
        }
        let n = self.nodes.pop().unwrap();
        if let Some(node) = self.nodes.last_mut() {
            node.add_child(n);
        }
        Ok(())
    }

    /// Clear the current AST and start a new one with the given node at the top-level
    pub fn start(&mut self, node: Node) {
        self.nodes.clear();
        self.nodes.push(node);
    }

    pub fn process(&self, process_fn: &dyn Fn(&Node) -> Node) -> Node {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            process_fn(&node)
        } else {
            process_fn(&self.nodes[0])
        }
    }

    pub fn to_string(&self) -> String {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            node.to_string()
        } else {
            self.nodes[0].to_string()
        }
    }

    // Closes all currently open nodes into new node but leaving the original state of the AST
    // unmodified.
    // This is like a snapshot of the current AST state, mainly useful for printing to the console
    // for debug.
    fn to_node(&self) -> Node {
        let mut node = self.nodes.last().unwrap().clone();
        let num = self.nodes.len();
        if num > 1 {
            for i in 1..num {
                let n = node;
                node = self.nodes[num - i - 1].clone();
                node.add_child(n);
            }
        }
        node
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

type Id = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attrs {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Test (patgen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// The top-level node type
    Test(String),
    Comment(String),
    PinWrite(Id, u128),
    PinVerify(Id, u128),
    RegWrite(Id, BigUint),
    RegVerify(Id, BigUint),
    Cycle(u32),
    //// Teradyne custom nodes

    //// Advantest custom nodes

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Flow (proggen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
}

#[derive(Clone, Debug)]
pub struct Node {
    pub attrs: Attrs,
    pub meta: Option<Meta>,
    // This must remain private, potentially we could run across some limitation of this current children
    // implementation which could force us to change to (for example) an ID based system instead.
    // Ensuring all interation with this collection is via an API method will allow us to make such a
    // change under-the-hood without breaking the world.
    children: Vec<Box<Node>>,
}

#[derive(Clone, Debug)]
pub struct Meta {
    filename: Option<String>,
    lineno: Option<usize>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Node {
    pub fn new(attrs: Attrs) -> Node {
        Node {
            attrs: attrs,
            children: Vec::new(),
            meta: None,
        }
    }

    pub fn new_with_meta(attrs: Attrs, filename: Option<String>, lineno: Option<usize>) -> Node {
        Node {
            attrs: attrs,
            children: Vec::new(),
            meta: Some(Meta {
                filename: filename,
                lineno: lineno,
            }),
        }
    }

    fn inline(nodes: Vec<Box<Node>>) -> Node {
        Node {
            attrs: Attrs::_Inline,
            meta: None,
            children: nodes,
        }
    }

    pub fn to_string(&self) -> String {
        ToString::run(self)
    }

    pub fn add_child(&mut self, node: Node) {
        self.children.push(Box::new(node));
    }

    /// Returns a new node which is the output of the node processed by the
    /// given processor.
    /// Returning None means that the processor has decided that the node should
    /// be deleted from the next stage AST.
    pub fn process(&self, processor: &mut dyn Processor) -> Option<Node> {
        // Call the dedicated handler for this node if it exists
        let r = match &self.attrs {
            Attrs::Test(name) => processor.on_test(&name, &self),
            Attrs::Comment(msg) => processor.on_comment(&msg, &self),
            Attrs::PinWrite(id, val) => processor.on_pin_write(*id, *val),
            Attrs::PinVerify(id, val) => processor.on_pin_verify(*id, *val),
            Attrs::RegWrite(id, val) => processor.on_reg_write(*id, val),
            Attrs::RegVerify(id, val) => processor.on_reg_verify(*id, val),
            Attrs::Cycle(repeat) => processor.on_cycle(*repeat, &self),
            _ => Return::Unimplemented,
        };
        // If not, call the default handler all nodes handler
        let r = match r {
            Return::Unimplemented => processor.on_all(&self),
            _ => r,
        };
        self.process_return_code(r, processor)
    }

    fn process_return_code(&self, code: Return, processor: &mut dyn Processor) -> Option<Node> {
        match code {
            Return::None => None,
            Return::ProcessChildren => Some(self.process_children(processor)),
            Return::Unmodified => Some(self.clone()),
            Return::Replace(node) => Some(node),
            // We can't return multiple nodes from this function, so we return them
            // wrapped in a meta-node and the process_children method will identify
            // this and remove the wrapper to inline the contained nodes.
            Return::Unwrap => Some(Node::inline(self.children.clone())),
            Return::Inline(nodes) => Some(Node::inline(nodes)),
            Return::InlineUnboxed(nodes) => Some(Node::inline(
                nodes.into_iter().map(|n| Box::new(n)).collect(),
            )),
            _ => None,
        }
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by their processed counterparts.
    pub fn process_children(&self, processor: &mut dyn Processor) -> Node {
        if self.children.len() == 0 {
            return self.clone();
        }
        let mut nodes: Vec<Box<Node>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor) {
                if let Attrs::_Inline = node.attrs {
                    for c in node.children {
                        nodes.push(c);
                    }
                } else {
                    nodes.push(Box::new(node));
                }
            }
        }
        // Call the end of block handler, giving the processor a chance to do any
        // internal clean up or inject some more nodes at the end
        let r = processor.on_end_of_block(&self);
        if let Some(node) = self.process_return_code(r, processor) {
            if let Attrs::_Inline = node.attrs {
                for c in node.children {
                    nodes.push(c);
                }
            } else {
                nodes.push(Box::new(node));
            }
        }
        self.replace_children(nodes)
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    pub fn replace_children(&self, nodes: Vec<Box<Node>>) -> Node {
        let new_node = Node {
            attrs: self.attrs.clone(),
            meta: self.meta.clone(),
            children: nodes,
        };
        new_node
    }

    /// Returns a new node which is a copy of self with its attrs replaced
    /// by the given attrs.
    pub fn replace_attrs(&self, attrs: Attrs) -> Node {
        let new_node = Node {
            attrs: attrs,
            meta: self.meta.clone(),
            children: self.children.clone(),
        };
        new_node
    }
}
