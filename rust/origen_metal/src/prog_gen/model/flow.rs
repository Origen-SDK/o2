use super::Bin;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Flow {
    pub tests: Vec<usize>,
    pub test_invocations: Vec<usize>,
    /// All hardbins referenced in the flow
    pub hardbins: IndexMap<usize, Bin>,
    /// All softbins referenced in the flow
    pub softbins: IndexMap<usize, Bin>,
    /// The IDs of all pattern references within this flow
    pub patterns: Vec<usize>,
    /// The IDs of all variable references within this flow
    pub variables: Vec<usize>,
}

impl Flow {
    pub fn new() -> Self {
        Self {
            tests: vec![],
            test_invocations: vec![],
            hardbins: IndexMap::new(),
            softbins: IndexMap::new(),
            patterns: vec![],
            variables: vec![],
        }
    }
}
