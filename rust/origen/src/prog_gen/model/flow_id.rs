/// Used to uniquely identify a test in a flow
#[derive(Clone, Serialize, PartialEq, Hash, Eq)]
pub struct FlowID {
    id: String,
    _private: (),
}

impl FlowID {
    /// Generate a new ID from a string. No checking is done at the point of creation
    /// to guarantee uniqueness, but it will be checked later in the generation process.
    /// String-based IDs are forced to lowercase to enable case-insensitive comparisions.
    pub fn from_str(id: &str) -> FlowID {
        FlowID {
            id: id.to_lowercase(),
            _private: (),
        }
    }

    /// Generate a new ID from an integer. No checking is done at the point of creation
    /// to guarantee uniqueness, but it will be checked later in the generation process.
    pub fn from_int(id: usize) -> FlowID {
        FlowID {
            id: format!("{}", id),
            _private: (),
        }
    }

    /// Generate a new unique ID
    pub fn new() -> FlowID {
        FlowID::from_int(crate::STATUS.generate_unique_id())
    }
}

impl std::fmt::Display for FlowID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlowID(\"{}\")", self.id)
    }
}

impl std::fmt::Debug for FlowID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlowID(\"{}\")", self.id)
    }
}
