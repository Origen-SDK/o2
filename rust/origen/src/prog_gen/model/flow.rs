use super::Bin;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Flow {
    /// All hardbins referenced in the flow
    pub hardbins: IndexMap<usize, Bin>,
    /// All softbins referenced in the flow
    pub softbins: IndexMap<usize, Bin>,
    /// The IDs of all pattern references within this flow
    pub patterns: Vec<usize>,
}

impl Flow {
    pub fn new() -> Self {
        Self {
            hardbins: IndexMap::new(),
            softbins: IndexMap::new(),
            patterns: vec![],
        }
    }
}
