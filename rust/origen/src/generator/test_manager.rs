//! Responsible for managing the current test (pattern).
//! This basically manages a RWLock over an AST representing the current test, allowing
//! application code to always deal with an immutable reference to an instance of this
//! struct at origen::TEST.

use crate::generator::ast::*;
use crate::Result;
use std::fmt;
use std::sync::RwLock;

pub struct TestManager {
    pub ast: RwLock<AST>,
}

impl TestManager {
    pub fn new() -> TestManager {
        let mut ast = AST::new();
        ast.push_and_open(node!(Test, "ad-hoc".to_string()));
        TestManager {
            ast: RwLock::new(ast),
        }
    }

    /// Starts a new test (deletes the current AST and starts a new one)
    pub fn start(&self, name: &str) {
        let mut ast = self.ast.write().unwrap();
        let node = node!(Test, name.to_string());
        ast.start(node);
    }

    /// Push a new terminal node into the AST
    pub fn push(&self, node: Node) {
        let mut ast = self.ast.write().unwrap();
        ast.push(node);
    }

    pub fn append(&self, nodes: &mut Vec<Node>) {
        let mut ast = self.ast.write().unwrap();
        ast.append(nodes);
    }

    /// Push a new node into the AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// A reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&self, node: Node) -> usize {
        let mut ast = self.ast.write().unwrap();
        ast.push_and_open(node)
    }

    /// Close the currently open node
    pub fn close(&self, ref_id: usize) -> Result<()> {
        let mut ast = self.ast.write().unwrap();
        ast.close(ref_id)
    }

    /// Replace the node n - offset with the given node, use offset = 0 to
    /// replace the last node that was pushed.
    /// Fails if the AST has no children yet or if the offset is otherwise out
    /// of range.
    pub fn replace(&self, node: Node, offset: usize) -> Result<()> {
        let mut ast = self.ast.write().unwrap();
        ast.replace(node, offset)
    }

    /// Returns a copy of node n - offset, an offset of 0 means
    /// the last node pushed.
    /// Fails if the offset is out of range.
    pub fn get(&self, offset: usize) -> Result<Node> {
        let ast = self.ast.write().unwrap();
        ast.get(offset)
    }

    /// Returns a copy of node n - offset, where an offset of 0 means
    /// the last node pushed.
    /// Differs from 'get' in that the offset will step into all nodes'
    /// children. For example, in the AST:
    ///     n1
    ///         n2
    ///             n2.1
    ///             n2.2
    ///         n3
    ///  offset | get(offset) | get_with_descendants(offset)
    ///  0      |       n3    |     n3
    ///  1      |       n2    |     n2.2
    ///  2      |       n1    |     n2.1
    ///
    /// Fails if the offset is out of range.
    pub fn get_with_descendants(&self, offset: usize) -> Result<Node> {
        let ast = self.ast.read().unwrap();
        ast.get_with_descendants(offset)
    }

    /// Insert the node at position n - offset, using offset = 0 is equivalent
    /// calling push().
    pub fn insert(&self, node: Node, offset: usize) -> Result<()> {
        let mut ast = self.ast.write().unwrap();
        ast.insert(node, offset)
    }

    pub fn to_string(&self) -> String {
        let ast = self.ast.read().unwrap();
        format!("{}", ast)
    }

    pub fn process(&self, process_fn: &mut dyn FnMut(&Node) -> Node) -> Node {
        let ast = self.ast.read().unwrap();
        ast.process(process_fn)
    }

    /// Execute the given function which receives the a reference to the AST (as a Node) as
    /// an input, returning the result of the function
    pub fn with_ast<T, F>(&self, mut process_fn: F) -> Result<T>
    where
        F: FnMut(&Node) -> Result<T>,
    {
        let ast = self.ast.read().unwrap();
        ast.with_node(&mut process_fn)
    }

    pub fn to_node(&self) -> Node {
        let ast = self.ast.read().unwrap();
        ast.to_node()
    }

    /// Serializes the AST for import into Python
    pub fn to_pickle(&self) -> Vec<u8> {
        let ast = self.ast.read().unwrap();

        ast.to_pickle()
    }
}

impl fmt::Display for TestManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ast.read().unwrap())
    }
}

impl fmt::Debug for TestManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ast.read().unwrap())
    }
}

impl PartialEq<AST> for TestManager {
    fn eq(&self, ast: &AST) -> bool {
        self.to_node() == ast.to_node()
    }
}

impl PartialEq<Node> for TestManager {
    fn eq(&self, node: &Node) -> bool {
        self.to_node() == *node
    }
}

impl Clone for TestManager {
    fn clone(&self) -> TestManager {
        let ast = self.ast.read().unwrap();
        TestManager {
            ast: RwLock::new(ast.clone()),
        }
    }
}
