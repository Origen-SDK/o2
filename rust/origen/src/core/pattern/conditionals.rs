//! Defines the if_true conditional structure
use id_arena::Arena;
use super::ast_node::{AstNode, AstNodeId};

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalIf {
    pub condition: AstNodeId,
    pub children: Arena::<AstNode>,
}

impl ConditionalIf {
    pub fn new(condition: &AstNodeId) -> ConditionalIf {
        ConditionalIf {
            condition: *condition,
            children: Arena::<AstNode>::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalElse {
    pub linked_if: AstNodeId,
    pub children: Arena::<AstNode>,
}

impl ConditionalElse {
    pub fn new(linked_if: &AstNodeId) -> ConditionalElse {
        ConditionalElse {
            linked_if: *linked_if,
            children: Arena::<AstNode>::new(),
        }
    }
}
