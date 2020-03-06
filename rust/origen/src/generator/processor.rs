//!
//! The processor API is intentionally placed in a separate modele from the AST/Node
//! to ensure that processor implementations use the Node API rather than coupling to its
//! internals (i.e. children vector) which could be subject to change.

use crate::generator::ast::*;
use num_bigint::BigUint;

/// All procesor handler methods should return this
pub enum Return {
    /// This is the value returned by the default Processor trait handlers
    /// and is used to indicated that a given processor has not implemented a
    /// handler for a given node type. Implementations of the Processor trait
    /// should never return this type.
    _Unimplemented,
    /// Deletes the node from the output AST.
    None,
    /// Clones the node (and all of its children) into the output AST. Note that
    /// the child nodes are not processed in this case (though they will appear in
    /// the output unmodified).
    Unmodified,
    /// Clones the node but replaces it's current children with their
    /// processed counterparts in the output AST.
    ProcessChildren,
    /// Replace the node in the output AST with the given node.
    Replace(Node),
    /// Removes the node and leaves its children in its place.
    Unwrap,
    /// Replace the node in the output AST with the given nodes, the vector wrapper
    /// will be removed and the nodes will be placed inline with where the current
    /// node is/was.
    Inline(Vec<Box<Node>>),
    /// Same as Inline, but accepts a vector of un-boxed nodes
    InlineUnboxed(Vec<Node>),
}

// Implements default handlers for all node types
pub trait Processor {
    /// This will be called for all nodes unless a dedicated handler
    /// handler exists for the given node type. It means that by default, all
    /// nodes will have their children processed by all processors.
    fn on_all(&mut self, _node: &Node) -> Return {
        Return::ProcessChildren
    }

    /// This will be called at the end of processing every node which has children.
    /// The node which is about to be closed is provided in the arguments.
    /// Note that you should probably never return a derivative of the given node
    /// here, it should either be None or a new node(s)
    fn on_end_of_block(&mut self, _node: &Node) -> Return {
        Return::None
    }

    fn on_test(&mut self, _name: &str, _node: &Node) -> Return {
        Return::_Unimplemented
    }

    fn on_comment(&mut self, _level: u8, _msg: &str, _node: &Node) -> Return {
        Return::_Unimplemented
    }

    fn on_pin_write(&mut self, _id: usize, _data: u128) -> Return {
        Return::_Unimplemented
    }

    fn on_pin_verify(&mut self, _id: usize, _data: u128) -> Return {
        Return::_Unimplemented
    }

    fn on_reg_write(
        &mut self,
        _id: usize,
        _data: &BigUint,
        _overlay_enable: &BigUint,
        _overlay_str: &Option<String>,
    ) -> Return {
        Return::_Unimplemented
    }

    fn on_reg_verify(
        &mut self,
        _id: usize,
        _val: &BigUint,
        _verify_enable: &BigUint,
        _capture_enable: &BigUint,
        _overlay_enable: &BigUint,
        _overlay_str: &Option<String>,
    ) -> Return {
        Return::_Unimplemented
    }

    fn on_cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Return {
        Return::_Unimplemented
    }

    fn on_flow(&mut self, _name: &str, _node: &Node) -> Return {
        Return::_Unimplemented
    }
}
