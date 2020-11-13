use super::template_loader::load_test_from_lib;
use super::{Bin, Limit, Test};
use crate::testers::SupportedTester;
use crate::Result;
use std::collections::HashMap;

/// The test program model contains tests, test invocations, patterns, bins, etc. that have been
/// extracted from a flow AST into a generic data structure that can be consumed by all tester
/// targets.
#[derive(Debug)]
pub struct Model {
    pub flow_name: String,
    pub tests: HashMap<usize, Test>,
    pub test_invocations: HashMap<usize, Test>,
    pub bins: HashMap<usize, Bin>,
    pub limits: HashMap<usize, Limit>,
    /// Templates which have been loaded into Test objects, organized by:
    ///   * Library Name
    ///     * Test Name
    templates: HashMap<String, HashMap<String, Test>>,
}

impl Model {
    pub fn new(flow_name: String) -> Self {
        Self {
            flow_name: flow_name,
            tests: HashMap::new(),
            test_invocations: HashMap::new(),
            bins: HashMap::new(),
            limits: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    /// Create a new test within the model from the given template reference.
    /// An error will be returned if the given template can not be found, or if a test alraedy
    /// exists with the given ID.
    pub fn add_test_from_template(
        &mut self,
        id: usize,
        name: String,
        tester: &SupportedTester,
        template_name: &str,
        library_name: Option<&str>,
    ) -> Result<()> {
        let library_name = match library_name {
            Some(d) => d,
            None => "std",
        };
        if !self.templates.contains_key(library_name) {
            self.templates
                .insert(library_name.to_string(), HashMap::new());
        }
        if let None = self.templates[library_name].get(template_name) {
            let mut test = Test::new(template_name, 0, tester.to_owned());

            if matches!(tester, SupportedTester::J750 | SupportedTester::ULTRAFLEX) {
                let base_template = load_test_from_lib(tester, "_internal", "test_instance")?;
                test.import_test_template(&base_template)?;
            }

            let test_template = load_test_from_lib(tester, library_name, template_name)?;
            test.import_test_template(&test_template)?;
            self.templates
                .get_mut(library_name)
                .unwrap()
                .insert(template_name.to_owned(), test);
        }
        let mut test = self.templates[library_name][template_name].clone();
        test.name = name;
        test.id = id;
        if self.tests.contains_key(&id) {
            error!("Something has gone wrong, two tests have been generated with the same internal ID in flow '{}': \nFirst:\n\n{:?}\n\nSecond:\n\n{:?}", &self.flow_name, &self.tests[&id], &test)
        } else {
            self.tests.insert(id, test);
            Ok(())
        }
    }

    /// Create a new test invocation within the model from the given tester reference.
    /// An error will be returned if a test invocation alraedy exists with the given ID.
    pub fn add_test_invocation(
        &mut self,
        id: usize,
        name: String,
        tester: &SupportedTester,
    ) -> Result<()> {
        if !self.templates.contains_key("_internal") {
            self.templates
                .insert("_internal".to_string(), HashMap::new());
        }
        let template_name = match tester {
            SupportedTester::J750 | SupportedTester::ULTRAFLEX => Some("flow_line"),
            SupportedTester::V93KSMT7 | SupportedTester::V93KSMT8 => Some("test_suite"),
            _ => None,
        };
        let test = match template_name {
            Some(template_name) => {
                if let None = self.templates["_internal"].get(template_name) {
                    let test_template = load_test_from_lib(tester, "_internal", template_name)?;
                    let mut t = Test::new(template_name, 0, tester.to_owned());
                    t.import_test_template(&test_template)?;
                    self.templates
                        .get_mut("_internal")
                        .unwrap()
                        .insert(template_name.to_owned(), t);
                }
                let mut test = self.templates["_internal"][template_name].clone();
                test.name = name;
                test.id = id;
                test
            }
            None => Test::new(&name, id, tester.to_owned()),
        };
        if self.test_invocations.contains_key(&id) {
            error!("Something has gone wrong, two test invocations have been generated with the same internal ID in flow '{}': \nFirst:\n\n{:?}\n\nSecond:\n\n{:?}", &self.flow_name, &self.tests[&id], &test)
        } else {
            self.test_invocations.insert(id, test);
            Ok(())
        }
    }
}
