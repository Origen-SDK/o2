use super::Limit;

/// SubTests are used to model a test method or IG-XL flow line which has multiple limits.
#[derive(Debug, Clone)]
pub struct SubTest {
    pub test_id: usize,
    /// If not present the name will be derived from the parent test
    pub name: Option<String>,
    /// Optional test number
    pub number: Option<usize>,
    pub lo_limit: Option<Limit>,
    pub hi_limit: Option<Limit>,
}

impl SubTest {
    pub fn new(
        test_id: usize,
        name: String,
        number: Option<usize>,
        lo_limit: Option<Limit>,
        hi_limit: Option<Limit>,
    ) -> Self {
        Self {
            test_id,
            name: Some(name),
            number,
            lo_limit,
            hi_limit,
        }
    }
}
