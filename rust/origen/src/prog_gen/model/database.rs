use super::{Bin, Limit, Test};
use crate::testers::SupportedTester;
use crate::Result;
use std::collections::HashMap;

/// The TestPrograms is a singleton which lives for the entire duration of an Origen program
/// generation run (the whole execution of an 'origen g' command), it is instantiated as
/// origen::PROG.
/// It provides long term storage for the test program model, similar to how the DUT provides long
/// term storage of the regs and other DUT models.
/// A complete test program model is maintained for each tester target, each model stores the test
/// templates and test instances for the given tester.
/// A common flow AST is shared by all testers and is therefore stored in this central data struct.
#[derive(Debug)]
pub struct Database {
    pub tests: Vec<Test>,
    pub bins: Vec<Bin>,
    pub limits: Vec<Limit>,
    /// Maps tests which are templates, organized as follows:
    /// * Supported Tester
    ///   * Library Name
    ///     * Test Name -> ID (reference to tests)
    libraries: HashMap<SupportedTester, HashMap<String, HashMap<String, usize>>>,
    /// This keeps track of what testers are selected by 'with specific tester' blocks in the application.
    /// It is implemented as a stack, allowing the application to select multiple testers at a time and then
    /// optionally select a subset via a nested with block.
    current_testers: Vec<Vec<SupportedTester>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tests: vec![],
            bins: vec![],
            limits: vec![],
            libraries: HashMap::new(),
            current_testers: vec![],
        }
    }

    pub fn push_current_testers(&mut self, testers: Vec<SupportedTester>) -> Result<()> {
        if testers.is_empty() {
            return error!("No tester type(s) given");
        }
        // When some testers are already selected, the application is only allowed to select a subset of them,
        // so verify that all given testers are already contained in the last selection
        if !self.current_testers.is_empty() {
            let last = self.current_testers.last().unwrap();
            for t in &testers {
                if !last.contains(t) {
                    return error!(
                        "Can't select tester '{}' within a block that already selects '{:?}'",
                        t, last
                    );
                }
            }
        }
        self.current_testers.push(testers.clone());
        Ok(())
    }

    pub fn pop_current_testers(&mut self) -> Result<()> {
        if self.current_testers.is_empty() {
            return error!("There has been an attempt to close a tester-specific block, but none is currently open in the program generator");
        }
        let _ = self.current_testers.pop();
        Ok(())
    }

    /// Creates a new test which is a duplicate of the given test, returning a mutable reference
    /// to the newly created test.
    /// The new test will be an exact duplicate of the original, including the value of the
    /// indirect flag (indirect means that it will not be rendered to the final test program).
    /// When creating a new test from a template use create_test_from_template instead, in that
    /// case the new test will be a duplicate except for its indirect flag which will be forced
    /// to false.
    pub fn create_duplicate_test(&mut self, parent_test_id: usize) -> Result<&mut Test> {
        let id = self.tests.len();
        match self.tests.get(parent_test_id) {
            None => error!(
                "Something has gone wrong, no test exists with ID '{}'",
                parent_test_id
            ),
            Some(t) => {
                let mut new_test = t.clone();
                new_test.id = id;
                self.tests.push(new_test);
                Ok(&mut self.tests[id])
            }
        }
    }

    /// Create a new test as a duplicate of the given template test, a mutable reference to the
    /// new test is returned.
    pub fn create_test_from_template(&mut self, parent_test_id: usize) -> Result<&mut Test> {
        let id = self.tests.len();
        match self.tests.get(parent_test_id) {
            None => error!(
                "Something has gone wrong, no test exists with ID '{}'",
                parent_test_id
            ),
            Some(t) => {
                let mut new_test = t.clone();
                new_test.id = id;
                new_test.indirect = false;
                self.tests.push(new_test);
                Ok(&mut self.tests[id])
            }
        }
    }

    /// Creates a test library with the given name, if it already exists no action will be taken
    pub fn create_test_library(&mut self, tester: &SupportedTester, name: &str) {
        if !self.libraries.contains_key(tester) {
            self.libraries.insert(tester.to_owned(), HashMap::new());
        }
        if !self.libraries[tester].contains_key(name) {
            self.libraries
                .get_mut(tester)
                .unwrap()
                .insert(name.to_string(), HashMap::new());
        }
    }

    /// Create a test with the given name within the given library, an error will be returned
    /// if the library doesn't exist or if a test with the same name already exists within the
    /// library.
    /// A mutable reference to the new test is returned to allow it to be further modified by the
    /// caller. Its ID and indirect fields are already set.
    pub fn create_test_template(
        &mut self,
        tester: &SupportedTester,
        library_name: &str,
        name: &str,
    ) -> Result<&mut Test> {
        if !self.libraries.contains_key(tester)
            && !self.libraries[tester].contains_key(library_name)
        {
            return error!(
                "A test library named '{}' does not exist for tester '{}'",
                library_name, tester
            );
        }

        let id = self.tests.len();
        let mut t = Test::new(name, id, tester.to_owned());
        t.indirect = true;

        // Add the new test to the library
        let lib = self
            .libraries
            .get_mut(tester)
            .unwrap()
            .get_mut(library_name)
            .unwrap();
        if !lib.contains_key(name) {
            return error!(
                "The test library '{}' for tester '{}' already contains a test template called '{}'",
                library_name, tester, name
            );
        } else {
            lib.insert(name.to_string(), id);
        }
        self.tests.push(t);
        Ok(&mut self.tests[id])
    }

    pub fn create_test(&mut self, tester: &SupportedTester, name: &str) -> &mut Test {
        let id = self.tests.len();
        let t = Test::new(name, id, tester.to_owned());
        self.tests.push(t);
        &mut self.tests[id]
    }

    pub fn get_test_template_id(
        &self,
        tester: &SupportedTester,
        library_name: &str,
        name: &str,
    ) -> Result<usize> {
        if !self.libraries.contains_key(tester)
            && !self.libraries[tester].contains_key(library_name)
        {
            return error!(
                "A test library named '{}' does not exist for tester '{}'",
                library_name, tester
            );
        }
        match self.libraries[tester][library_name].get(name) {
            None => error!(
                "The test library '{}' for tester '{}' does not contain a test template called '{}'",
                library_name, tester, name
            ),
            Some(y) => Ok(y.to_owned()),
        }
    }
}
