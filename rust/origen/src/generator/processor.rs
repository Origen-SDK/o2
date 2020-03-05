//!
//! The processor API is intentionally placed in a separate modele from the AST/Node
//! to ensure that processor implementations use the Node API rather than coupling to its
//! internals (i.e. children vector) which could be subject to change.

use crate::generator::ast::*;
use num_bigint::BigUint;

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
pub trait Processor {
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

    fn on_pin_write(&mut self, _id: usize, _val: u128) -> Return {
        Return::Unimplemented
    }

    fn on_pin_verify(&mut self, _id: usize, _val: u128) -> Return {
        Return::Unimplemented
    }

    fn on_reg_write(&mut self, _id: usize, _val: &BigUint) -> Return {
        Return::Unimplemented
    }

    fn on_reg_verify(&mut self, _id: usize, _val: &BigUint) -> Return {
        Return::Unimplemented
    }

    fn on_cycle(&mut self, _repeat: u32, _node: &Node) -> Return {
        Return::Unimplemented
    }
}
