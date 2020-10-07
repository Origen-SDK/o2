#[derive(Debug)]
pub struct TestCollection {
    pub name: String,
    pub test_ids: Vec<usize>,
}

impl TestCollection {
    pub fn new(name: &str) -> TestCollection {
        TestCollection {
            name: name.to_string(),
            test_ids: vec![],
        }
    }
}
