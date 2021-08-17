use super::Limit;

/// SubTests are used to model a test method or IG-XL flow line which has multiple limits.
#[derive(Debug)]
pub struct SubTest {
    test_id: usize,
    /// If not present the name will be derived from the parent test
    name: Option<String>,
    /// Optional test number
    number: Option<usize>,
    lo_limit: Option<Limit>,
    hi_limit: Option<Limit>,
}
