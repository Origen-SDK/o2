use super::Limit;

/// SubTests are used to model a test method or IG-XL flow line which has multiple limits.
#[derive(Debug)]
pub struct SubTest {
    #[allow(dead_code)]
    test_id: usize,
    /// If not present the name will be derived from the parent test
    #[allow(dead_code)]
    name: Option<String>,
    /// Optional test number
    #[allow(dead_code)]
    number: Option<usize>,
    #[allow(dead_code)]
    lo_limit: Option<Limit>,
    #[allow(dead_code)]
    hi_limit: Option<Limit>,
}
