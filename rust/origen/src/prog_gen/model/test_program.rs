use super::{Test, TestCollection, TestInvocation};
use crate::generator::ast::AST;
use crate::Result;
use std::collections::HashMap;

/// The TestProgram is a singleton which lives for the entire duration of an Origen program
/// generation run (the whole execution of an 'origen g' command), it is instantiated as
/// origen::PROG.
/// It provides long term storage for test obects, similar to how the DUT provides long
/// term storage of the regs and other DUT models.
pub struct TestProgram {
    tests: Vec<Test>,
    test_invocations: Vec<TestInvocation>,
    test_collections: Vec<TestCollection>,
    /// Flows are represented as an AST, which will contain references to tests from the
    /// above collection
    flows: HashMap<String, AST>,
    /// Maps tests which are templates
    templates: HashMap<String, HashMap<String, usize>>,
    current_collection: usize,
}

impl TestProgram {
    pub fn new() -> Self {
        Self {
            tests: vec![],
            test_invocations: vec![],
            // New tests will be put into the global collection by default
            test_collections: vec![TestCollection::new("global")],
            flows: HashMap::new(),
            templates: HashMap::new(),
            current_collection: 0,
        }
    }

    /// Get a read-only reference to the test with the given ID, use get_mut_test if
    /// you need to modify it
    pub fn get_test(&self, id: usize) -> Result<&Test> {
        match self.tests.get(id) {
            Some(x) => Ok(x),
            None => error!(
                "Something has gone wrong, no register exists with ID '{}'",
                id
            ),
        }
    }

    pub fn add_test_library(&mut self, name: &str) -> Result<usize> {
        if self.templates.contains_key(name) {
            error!("A test library named '{}' already exists!", name)
        } else {
            let id = self.templates.len();
            self.templates.insert(name.to_string(), HashMap::new());
            Ok(id)
        }
    }

    pub fn add_test_template<T, F>(
        &mut self,
        library_name: &str,
        name: &str,
        mut func: F,
    ) -> Result<T>
    where
        F: FnMut(&mut Test, &TestProgram) -> Result<T>,
    {
        let id = self.tests.len();
        let mut t = Test::new(name, id);
        t.indirect = true;
        // Add the new test to the library
        match self.templates.get_mut(library_name) {
            Some(x) => match x.contains_key(name) {
                true => {
                    return error!(
                        "The test library '{}' already contains a test template called '{}'",
                        library_name, name
                    )
                }
                false => x.insert(name.to_string(), id),
            },
            None => return error!("A test library named '{}' does not exist", library_name),
        };
        let result = func(&mut t, self);
        self.tests.push(t);
        result
    }

    pub fn add_test<T, F>(&mut self, name: &str, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Test, &TestProgram) -> Result<T>,
    {
        let id = self.tests.len();
        let mut t = Test::new(name, id);
        let result = func(&mut t, self);
        self.tests.push(t);
        result
    }

    pub fn get_test_template_id(&self, library_name: &str, name: &str) -> Result<usize> {
        match self.templates.get(library_name) {
            Some(x) => match x.get(name) {
                None => error!(
                    "The test library '{}' does not contain a test template called '{}'",
                    library_name, name
                ),
                Some(y) => Ok(y.to_owned()),
            },
            None => error!("A test library named '{}' does not exist", library_name),
        }
    }
}
