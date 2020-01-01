//! Defines the if_true conditional structure
use super::ast_node::AstNodeId;

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalIf {
    pub condition: AstNodeId,
    pub children: Vec<AstNodeId>,
}

impl ConditionalIf {
    pub fn new(condition: &AstNodeId) -> ConditionalIf {
        ConditionalIf {
            condition: *condition,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalElse {
    pub linked_if: AstNodeId,
    pub children: Vec<AstNodeId>,
}

impl ConditionalElse {
    pub fn new(linked_if: &AstNodeId) -> ConditionalElse {
        ConditionalElse {
            linked_if: *linked_if,
            children: Vec::new(),
        }
    }
}
