pub use super::node::Meta;
pub use super::node::{Attrs, Node};
//use crate::generator::TestManager;
//use crate::TEST;
use crate::{Error, Result};
use std::fmt;

#[macro_export]
macro_rules! push_pin_actions {
    ($pin_info:expr) => {{
        crate::TEST.push(crate::node!(PinAction, $pin_info));
    }};
}

#[macro_export]
macro_rules! text {
    ($txt:expr) => {{
        crate::node!(Text, $txt.to_string())
    }};
}

#[macro_export]
macro_rules! add_children {
    ( $parent:expr, $( $child:expr ),* ) => {{
        let mut p = $parent;
        $( p.add_child($child); )*
        p
    }};
}

#[macro_export]
macro_rules! text_line {
    ( $( $elem:expr ),* ) => {{
        let mut n = node!(TextLine);
        $( n.add_child($elem); )*
        n
    }};
}

/// An AST provides an API for constructing a node tree, when completed it can be unwrapped
/// to a node by calling the unwrap() method
#[derive(Clone)]
pub struct AST<T> {
    nodes: Vec<Node<T>>,
}

impl<T: Attrs> AST<T> {
    /// Create a new AST with the given node as the top-level
    pub fn new() -> AST<T> {
        AST { nodes: vec![] }
    }

    /// Consumes the AST, converting it to a Node
    pub fn unwrap(&mut self) -> Node<T> {
        while self.nodes.len() > 1 {
            let n = self.nodes.pop().unwrap();
            if let Some(node) = self.nodes.last_mut() {
                node.add_child(n);
            }
        }
        self.nodes.pop().unwrap()
    }

    /// Push a new terminal node into the AST
    pub fn push(&mut self, node: Node<T>) {
        match self.nodes.last_mut() {
            Some(n) => n.add_child(node),
            None => self.nodes.push(node),
        }
    }

    pub fn append(&mut self, nodes: &mut Vec<Node<T>>) {
        match self.nodes.last_mut() {
            Some(n) => {
                n.add_children(nodes.to_vec());
            }
            None => self.nodes.append(nodes),
        }
    }

    /// Push a new node into the AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// A reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of AST application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&mut self, node: Node<T>) -> usize {
        self.nodes.push(node);
        self.nodes.len()
    }

    /// Close the currently open node
    pub fn close(&mut self, ref_id: usize) -> Result<()> {
        if self.nodes.len() != ref_id {
            return Err(Error::new(&format!(
                "Attempt to close a parent AST node without first closing all its children (given ID: {}, current length: {}), it looks like you have either forgotten to close an open node or given the wrong reference ID",
                ref_id, self.nodes.len()
            )));
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

    /// Replace the node n - offset with the given node, use offset = 0 to
    /// replace the last node that was pushed.
    /// Fails if the AST has no children yet or if the offset is otherwise out
    /// of range.
    pub fn replace(&mut self, node: Node<T>, mut offset: usize) -> Result<()> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let mut root_node = false;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node to be replaced lies in this node's children
            if num_children > offset {
                child_offset = offset;
                offset = 0;
            // If node to be replaced is this node itself
            } else if num_children == offset {
                root_node = true;
                offset = 0;
            // The node to be replaced lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let mut n = self.nodes.remove(index);
        if root_node {
            self.nodes.insert(index, node);
        } else {
            n.replace_child(node, child_offset)?;
            self.nodes.insert(index, n);
        }
        Ok(())
    }

    /// Insert the node at position n - offset, using offset = 0 is equivalent
    /// calling push().
    pub fn insert(&mut self, node: Node<T>, mut offset: usize) -> Result<()> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node is to be inserted into this node's children
            if num_children >= offset {
                child_offset = offset;
                offset = 0;
            // The parent node lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let mut n = self.nodes.remove(index);
        n.insert_child(node, child_offset)?;
        self.nodes.insert(index, n);
        Ok(())
    }

    /// Returns a copy of node n - offset, an offset of 0 means
    /// the last node pushed.
    /// Fails if the offset is out of range.
    pub fn get(&self, mut offset: usize) -> Result<Node<T>> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let mut root_node = false;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node to be returned lies in this node's children
            if num_children > offset {
                child_offset = offset;
                offset = 0;
            // If node to be returned is this node itself
            } else if num_children == offset {
                root_node = true;
                offset = 0;
            // The node to be returned lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let n = &self.nodes[index];
        if root_node {
            Ok(self.nodes[index].clone())
        } else {
            Ok(n.get_child(child_offset)?)
        }
    }

    pub fn get_with_descendants(&self, offset: usize) -> Result<Node<T>> {
        let mut cnt: usize = 0;
        for n in self.nodes.iter().rev() {
            if let Some(node) = n.get_descendant(offset, &mut cnt) {
                return Ok(node);
            }
        }
        Err(Error::new(&format!(
            "Offset {} is out of range of the current AST",
            offset
        )))
    }

    /// Clear the current AST and start a new one with the given node at the top-level
    pub fn start(&mut self, node: Node<T>) {
        self.nodes.clear();
        self.nodes.push(node);
    }

    pub fn process(
        &self,
        process_fn: &mut dyn FnMut(&Node<T>) -> Result<Node<T>>,
    ) -> Result<Node<T>> {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            process_fn(&node)
        } else {
            process_fn(&self.nodes[0])
        }
    }

    /// Execute the given function which receives the a reference to the AST (as a Node) as
    /// its input, returning the result of the function
    pub fn with_node<N, F>(&self, mut process_fn: F) -> Result<N>
    where
        F: FnMut(&Node<T>) -> Result<N>,
    {
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

    /// Serializes the AST for import into Python
    pub fn to_pickle(&self) -> Vec<u8> {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            node.to_pickle()
        } else {
            self.nodes[0].to_pickle()
        }
    }

    /// Clones the current state of the AST into a Node, leaving the AST unmodified
    pub fn to_node(&self) -> Node<T> {
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

impl<T> fmt::Display for AST<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<T> fmt::Debug for AST<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<T: Attrs> PartialEq<Node<T>> for AST<T> {
    fn eq(&self, node: &Node<T>) -> bool {
        self.to_node() == *node
    }
}

//impl<T> PartialEq<TEST> for AST<T> {
//    fn eq(&self, test: &TEST) -> bool {
//        self.to_node() == test.to_node()
//    }
//}
//
//impl<T> PartialEq<TestManager> for AST<T> {
//    fn eq(&self, test: &TestManager) -> bool {
//        self.to_node() == test.to_node()
//    }
//}
