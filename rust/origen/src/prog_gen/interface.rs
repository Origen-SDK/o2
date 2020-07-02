use super::test::{Definition, Test};
use crate::generator::ast::AST;
use std::collections::HashMap;

/// The interface is a singleton which lives for the entire duration of an Origen program
/// generation run (the whole execution of an 'origen g' command), it is instantiated as
/// origen::INTERFACE.
/// It provides long term storage for test obects, similar to how the DUT provides long
/// term storage of the regs and other DUT models.
pub struct Interface {
    /// Contains all tests referenced in all flows, accessible by their ID which is their
    /// index number
    _tests: Vec<Test>,
    _flows: HashMap<String, AST>,
    _test_definitions: Vec<Definition>,
}

impl Interface {
    pub fn new() -> Self {
        Self {
            _tests: vec![],
            _flows: HashMap::new(),
            _test_definitions: vec![],
        }
    }
}
