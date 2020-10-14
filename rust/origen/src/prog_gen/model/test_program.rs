use super::{Test, TestCollection, TestInvocation};
use crate::generator::ast::AST;
use crate::testers::SupportedTester;
use crate::Result;
use std::collections::HashMap;
use std::sync::RwLock;

/// One test program model represents the test program for one tester target
#[derive(Debug)]
pub struct TestProgram {
    tests: RwLock<Vec<Test>>,
    test_invocations: RwLock<Vec<TestInvocation>>,
    test_collections: RwLock<Vec<TestCollection>>,
    /// Flows are represented as an AST, which will contain references to tests from the
    /// above collection
    flows: RwLock<Vec<AST>>,
    pointers: RwLock<Pointers>,
}

#[derive(Debug)]
struct Pointers {
    /// Maps names to a flow
    flow_map: HashMap<String, usize>,
    /// Maps tests which are templates, organized into named groups (test libraries)
    libraries: HashMap<String, HashMap<String, usize>>,
    current_collection: usize,
    current_flow: usize,
    /// Not known at creation time, but will be assigned before the first test is added
    /// to the program
    tester: SupportedTester,
}

impl TestProgram {
    pub fn new() -> Self {
        Self {
            tests: RwLock::new(vec![]),
            test_invocations: RwLock::new(vec![]),
            // New tests will be put into the global collection by default
            test_collections: RwLock::new(vec![TestCollection::new("global")]),
            flows: RwLock::new(vec![]),
            pointers: RwLock::new(Pointers {
                flow_map: HashMap::new(),
                libraries: HashMap::new(),
                current_collection: 0,
                current_flow: 0,
                tester: SupportedTester::CUSTOM("TBD".to_string()),
            }),
        }
    }

    pub fn set_tester(&self, tester: &SupportedTester) {
        let mut pointers = self.pointers.write().unwrap();
        pointers.tester = tester.to_owned();
    }

    /// Creates a new test which is a duplicate of the given test, returning the ID of the newly
    /// created test.
    /// The new test will be an exact duplicate of the original, including the value of the
    /// indirect flag (indirect means that it will not be rendered to the final test program).
    /// When creating a new test from a template use create_test_from_template instead, in that
    /// case the new test will be a duplicate except for its indirect flag which will be forced
    /// to false.
    pub fn create_duplicate_test(&self, parent_test_id: usize) -> Result<usize> {
        let mut tests = self.tests.write().unwrap();
        let id = tests.len();
        match tests.get(parent_test_id) {
            None => error!(
                "Something has gone wrong, no test exists with ID '{}'",
                parent_test_id
            ),
            Some(t) => {
                let mut new_test = t.clone();
                new_test.id = id;
                tests.push(new_test);
                Ok(id)
            }
        }
    }

    /// Create a new test as a duplicate of the given template test, the ID of the new test is
    /// returned.
    pub fn create_test_from_template(&self, parent_test_id: usize) -> Result<usize> {
        let mut tests = self.tests.write().unwrap();
        let id = tests.len();
        match tests.get(parent_test_id) {
            None => error!(
                "Something has gone wrong, no test exists with ID '{}'",
                parent_test_id
            ),
            Some(t) => {
                let mut new_test = t.clone();
                new_test.id = id;
                new_test.indirect = false;
                tests.push(new_test);
                Ok(id)
            }
        }
    }

    /// Get a read-only reference to the test with the given ID, use with_test_mut if
    /// you need to modify it.
    /// Returns an error if there is no test found, otherwise the result of the given function.
    pub fn with_test<T, F>(&self, id: usize, mut func: F) -> Result<T>
    where
        F: FnMut(&Test) -> Result<T>,
    {
        let tests = self.tests.read().unwrap();
        match tests.get(id) {
            Some(x) => func(x),
            None => error!("Something has gone wrong, no test exists with ID '{}'", id),
        }
    }

    /// Get a writable reference to the test with the given ID.
    /// Returns an error if there is no test found, otherwise the result of the given function.
    pub fn with_test_mut<T, F>(&self, id: usize, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Test) -> Result<T>,
    {
        let mut tests = self.tests.write().unwrap();
        match tests.get_mut(id) {
            Some(x) => func(x),
            None => error!("Something has gone wrong, no test exists with ID '{}'", id),
        }
    }

    /// Creates a test library with the given name, if it already exists no action will be taken
    pub fn create_test_library(&self, name: &str) {
        let mut pointers = self.pointers.write().unwrap();
        if !pointers.libraries.contains_key(name) {
            pointers.libraries.insert(name.to_string(), HashMap::new());
        }
    }

    pub fn create_test_template<F>(
        &self,
        library_name: &str,
        name: &str,
        mut func: F,
    ) -> Result<usize>
    where
        F: FnMut(&mut Test) -> Result<()>,
    {
        let mut tests = self.tests.write().unwrap();
        let mut pointers = self.pointers.write().unwrap();
        let id = tests.len();
        let mut t = Test::new(name, id, pointers.tester.clone());
        t.indirect = true;
        // Add the new test to the library
        match pointers.libraries.get_mut(library_name) {
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
        func(&mut t)?;
        tests.push(t);
        Ok(id)
    }

    pub fn create_test<T, F>(&mut self, name: &str, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Test) -> Result<T>,
    {
        let mut tests = self.tests.write().unwrap();
        let id = tests.len();
        let mut t = Test::new(name, id, self.pointers.read().unwrap().tester.clone());
        let result = func(&mut t);
        tests.push(t);
        result
    }

    pub fn get_test_template_id(&self, library_name: &str, name: &str) -> Result<usize> {
        let pointers = self.pointers.read().unwrap();
        match pointers.libraries.get(library_name) {
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
