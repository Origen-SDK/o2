//! Responsible for managing the current test (pattern).

use crate::generator::ast::*;
use crate::Result;
use std::fmt;
use std::sync::RwLock;

#[derive(Debug)]
pub struct TestManager {
    pub ast: RwLock<AST>,
}

impl TestManager {
    pub fn new() -> TestManager {
        TestManager {
            ast: RwLock::new(AST::new(Node::new(Attrs::Test("ad-hoc".to_string())))),
        }
    }

    /// Starts a new test (deletes the current AST and starts a new one)
    pub fn start(&self, name: &str) {
        let mut ast = self.ast.write().unwrap();
        let node = Node::new(Attrs::Test(name.to_string()));
        ast.start(node);
    }

    /// Push a new terminal node into the AST
    pub fn push(&self, node: Node) {
        let mut ast = self.ast.write().unwrap();
        ast.push(node);
    }

    /// Push a new node into the AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// An reference ID is returned and the caller should save this and provide it again
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

    pub fn to_string(&self) -> String {
        let ast = self.ast.read().unwrap();
        format!("{}", ast)
    }
}

impl fmt::Display for TestManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ast.read().unwrap())
    }
}
