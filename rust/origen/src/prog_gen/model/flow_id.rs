/// Used to uniquely identify a test in a flow
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct FlowID {
    id: String,
}

impl FlowID {
    /// Generate a new ID from a string. No checking is done at the point of creation
    /// to guarantee uniqueness, but it will be checked later in the generation process.
    pub fn from_str(id: &str) -> FlowID {
        FlowID { id: id.to_owned() }
    }

    /// Generate a new ID from an integer. No checking is done at the point of creation
    /// to guarantee uniqueness, but it will be checked later in the generation process.
    pub fn from_int(id: usize) -> FlowID {
        FlowID {
            id: format!("{}", id),
        }
    }

    /// Generate a new unique ID
    pub fn new() -> FlowID {
        FlowID::from_int(crate::STATUS.generate_unique_id())
    }
}
