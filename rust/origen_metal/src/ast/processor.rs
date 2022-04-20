//! The processor API is intentionally placed in a separate modele from the AST/Node
//! to ensure that processor implementations use the Node API rather than coupling to its
//! internals (i.e. children vector) which could be subject to change.

//use crate::generator::ast::*;
use crate::ast::node::Node;
pub use crate::Result;

/// All procesor handler methods should return this
pub enum Return<T> {
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
    Replace(Node<T>),
    /// Removes the node and leaves its children in its place.
    Unwrap,
    /// Replace the node in the output AST with the given nodes, the vector wrapper
    /// will be removed and the nodes will be placed inline with where the current
    /// node is/was.
    Inline(Vec<Node<T>>),
    /// Same as Inline, but accepts a vector of boxed nodes
    InlineBoxed(Vec<Box<Node<T>>>),
    /// A combination of Unwrap and ProcessChildren, which will unwrap the current node
    /// but leave processed children in its place
    UnwrapWithProcessedChildren,
    /// A combination of Inline and ProcessChildren which will add the given nodes
    /// then proceed to process the original node's children
    InlineWithProcessedChildren(Vec<Node<T>>),
    /// Keep the node in the new AST but replace its children with the given nodes
    ReplaceChildren(Vec<Node<T>>),
}

pub trait Processor<T> {
    fn on_node(&mut self, _node: &Node<T>) -> Result<Return<T>> {
        Ok(Return::ProcessChildren)
    }

    /// This will be called at the end of processing every node which has children.
    /// The node which is about to be closed is provided in the arguments.
    /// Note that you should probably never return a derivative of the given node
    /// here, it should either be None or a new node(s)
    fn on_end_of_block(&mut self, _node: &Node<T>) -> Result<Return<T>> {
        Ok(Return::None)
    }
}
