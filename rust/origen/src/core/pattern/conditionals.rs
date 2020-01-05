//! Defines the if_true conditional structure
use super::ast_node::AstNodeId;
use super::collector::Collector;

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalIf {
    pub condition: AstNodeId,
    pub children: Collector,
}

impl ConditionalIf {
    pub fn new(condition: &AstNodeId) -> ConditionalIf {
        ConditionalIf {
            condition: *condition,
            children: Collector::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConditionalElse {
    pub linked_if: AstNodeId,
    pub children: Collector,
}

impl ConditionalElse {
    pub fn new(linked_if: &AstNodeId) -> ConditionalElse {
        ConditionalElse {
            linked_if: *linked_if,
            children: Collector::new(),
        }
    }
}
