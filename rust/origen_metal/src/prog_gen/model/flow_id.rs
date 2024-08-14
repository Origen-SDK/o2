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
            id: format!("t{}", id),
            _private: (),
        }
    }

    /// Generate a new unique ID, this will have the format "_t{unique_id}_" which should
    /// avoid collisions with user-defined IDs
    pub fn new() -> FlowID {
        FlowID::from_str(&format!("_t{}_", crate::PROG_GEN_CONFIG.generate_unique_id()))
    }

    /// Returns true if the ID refers to a test external to this flow, currently defined by the ID
    /// starting with "extern_"
    pub fn is_external(&self) -> bool {
        self.id.starts_with("extern")
    }

    pub fn to_string(&self) -> String {
        self.id.clone()
    }

    pub fn to_str(&self) -> &str {
        &self.id
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
