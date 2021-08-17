use super::template_loader::load_test_from_lib;
use super::{
    Flow, ParamValue, Pattern, PatternReferenceType, PatternType, ResourcesType, SubTest, Test,
    Variable, VariableOperation, VariableType,
};
use crate::testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
use std::collections::HashSet;

/// The test program model contains tests, test invocations, patterns, bins, etc. that have been
/// extracted from a flow AST into a generic data structure that can be consumed by all tester
/// targets.
#[derive(Debug)]
pub struct Model {
    pub tester: SupportedTester,
    /// Test objects, stored by their internal ID.
    /// These map to test instances for IG-XL and test methods for V93K.
    pub tests: IndexMap<usize, Test>,
    /// Test invocation objects, stored by their internal ID.
    /// These map to test flow lines for IG-XL and test suites for V93K.
    pub test_invocations: IndexMap<usize, Test>,
    /// Tests can store a single limit, but if a test has multiple limits then they are represented as sub-tests
    pub sub_tests: Vec<SubTest>,
    /// All pattern references made in the test program, flows and pattern_collections make reference to these
    /// via their ID (their vector index number)
    pub patterns: Vec<Pattern>,
    /// All variable references made in the test program, flows and variable_collections make reference to these
    /// via their ID (their vector index number)
    pub variables: Vec<Variable>,
    pub flows: IndexMap<String, Flow>,
    pub pattern_collections: IndexMap<String, Vec<usize>>,
    pub variable_collections: IndexMap<String, Vec<usize>>,
    /// Templates which have been loaded into Test objects, organized by:
    ///   * Library Name
    ///     * Test Name
    templates: IndexMap<String, IndexMap<String, Test>>,
    current_flow: String,
    current_resource: String,
    current_pattern_resource: Option<String>,
    current_variable_resource: Option<String>,
}

impl Model {
    pub fn new(tester: SupportedTester) -> Self {
        Self {
            tester: tester,
            current_flow: "".to_string(),
            current_resource: "global".to_string(),
            current_pattern_resource: None,
            current_variable_resource: None,
            tests: IndexMap::new(),
            test_invocations: IndexMap::new(),
            sub_tests: vec![],
            templates: IndexMap::new(),
            patterns: vec![],
            variables: vec![],
            flows: IndexMap::new(),
            pattern_collections: IndexMap::new(),
            variable_collections: IndexMap::new(),
        }
    }

    pub fn set_resources_filename(&mut self, name: String, kind: &ResourcesType) {
        match kind {
            ResourcesType::All => {
                self.current_resource = name;
                self.current_pattern_resource = None;
                self.current_variable_resource = None;
            }
            ResourcesType::Patterns => {
                self.current_pattern_resource = Some(name);
            }
            ResourcesType::Variables => {
                self.current_variable_resource = Some(name);
            }
        }
    }

    pub fn patterns_from_ids(&self, ids: &Vec<usize>, sort: bool, uniq: bool) -> Vec<&Pattern> {
        let mut pats: Vec<&Pattern> = ids.iter().map(|id| &self.patterns[*id]).collect();
        if uniq {
            let mut existing = HashSet::new();
            pats.retain(|&p| {
                if existing.contains(p) {
                    false
                } else {
                    existing.insert(p);
                    true
                }
            });
        }
        if sort {
            pats.sort_by_key(|p| &p.path);
        }
        pats
    }

    pub fn variables_from_ids(&self, ids: &Vec<usize>, sort: bool, uniq: bool) -> Vec<&Variable> {
        let mut vars: Vec<&Variable> = ids.iter().map(|id| &self.variables[*id]).collect();
        if uniq {
            let mut existing = HashSet::new();
            vars.retain(|&v| {
                if existing.contains(v) {
                    false
                } else {
                    existing.insert(v);
                    true
                }
            });
        }
        if sort {
            vars.sort_by_key(|v| &v.name);
        }
        vars
    }

    /// Set the current flow (default flow operated on by some of the model's methods), returns an error
    /// if the model doesn't contain a flow with the given name
    pub fn select_flow(&mut self, name: &str) -> Result<()> {
        if !self.flows.contains_key(name) {
            return error!("The test program doesn't contains a flow called '{}'", name);
        }
        self.current_flow = name.to_string();
        Ok(())
    }

    /// Creates a new flow within the model and selects it as the current flow.
    /// An error will be returned if a flow of the given name already exists.
    pub fn create_flow(&mut self, name: &str) -> Result<()> {
        let flow = Flow::new();
        if self.flows.contains_key(name) {
            return error!(
                "The test program model already contains a flow called '{}'",
                name
            );
        }
        self.flows.insert(name.to_string(), flow);
        self.current_flow = name.to_string();
        Ok(())
    }

    /// Get a reference to the current or given flow
    pub fn get_flow(&self, name: Option<&str>) -> Option<&Flow> {
        let name = match name {
            Some(n) => n,
            None => &self.current_flow,
        };
        self.flows.get(name)
    }

    /// Get a mutable reference to the current or given flow, will create it if it doesn't exist yet
    pub fn get_flow_mut(&mut self, name: Option<&str>) -> &mut Flow {
        let name = match name {
            Some(n) => n,
            None => &self.current_flow,
        };
        if !self.flows.contains_key(name) {
            self.flows.insert(name.to_string(), Flow::new());
        }
        self.flows.get_mut(name).unwrap()
    }

    pub fn current_pattern_collection_name(&self) -> &str {
        match &self.current_pattern_resource {
            Some(n) => n,
            None => &self.current_resource,
        }
    }

    /// Get a reference to the current or given pattern collection
    pub fn get_pattern_collection(&self, name: Option<&str>) -> Option<&Vec<usize>> {
        let name = match name {
            Some(n) => n,
            None => self.current_pattern_collection_name(),
        };
        self.pattern_collections.get(name)
    }

    /// Get a mutable reference to the current or given resource, will create it if it doesn't exist yet
    pub fn get_pattern_collection_mut(&mut self, name: Option<&str>) -> &mut Vec<usize> {
        let name = match name {
            Some(n) => n,
            // Had to inline current_pattern_collection_name here to satisfy the borrow checker
            None => match &self.current_pattern_resource {
                Some(n) => n,
                None => &self.current_resource,
            },
        };
        if !self.pattern_collections.contains_key(name) {
            self.pattern_collections.insert(name.to_string(), vec![]);
        }
        self.pattern_collections.get_mut(name).unwrap()
    }

    /// Record a pattern reference and allocate to the current flow and pattern collection
    pub fn record_pattern_reference(
        &mut self,
        path: String,
        pattern_type: Option<PatternType>,
        reference_type: Option<PatternReferenceType>,
    ) {
        let p = Pattern::new(path, pattern_type, reference_type);
        let id = self.patterns.len();
        self.patterns.push(p);
        let flow = self.get_flow_mut(None);
        flow.patterns.push(id);
        self.get_pattern_collection_mut(None).push(id);
    }

    pub fn current_variable_collection_name(&self) -> &str {
        match &self.current_variable_resource {
            Some(n) => n,
            None => &self.current_resource,
        }
    }

    /// Get a reference to the current or given variable collection
    pub fn get_variable_collection(&self, name: Option<&str>) -> Option<&Vec<usize>> {
        let name = match name {
            Some(n) => n,
            None => self.current_variable_collection_name(),
        };
        self.variable_collections.get(name)
    }

    /// Get a mutable reference to the current or given resource, will create it if it doesn't exist yet
    pub fn get_variable_collection_mut(&mut self, name: Option<&str>) -> &mut Vec<usize> {
        let name = match name {
            Some(n) => n,
            // Had to inline current_variable_collection_name here to satisfy the borrow checker
            None => match &self.current_variable_resource {
                Some(n) => n,
                None => &self.current_resource,
            },
        };
        if !self.variable_collections.contains_key(name) {
            self.variable_collections.insert(name.to_string(), vec![]);
        }
        self.variable_collections.get_mut(name).unwrap()
    }

    /// Record a variable reference and allocate to the current flow and variable collection
    pub fn record_variable_reference(
        &mut self,
        name: String,
        variable_type: VariableType,
        operation: VariableOperation,
    ) {
        let v = Variable::new(name, variable_type, operation);
        let id = self.variables.len();
        self.variables.push(v);
        let flow = self.get_flow_mut(None);
        flow.variables.push(id);
        self.get_variable_collection_mut(None).push(id);
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
                .insert(library_name.to_string(), IndexMap::new());
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
            error!("Something has gone wrong, two tests have been generated with the same internal ID in flow '{}': \nFirst:\n\n{:?}\n\nSecond:\n\n{:?}", &self.current_flow, &self.tests[&id], &test)
        } else {
            self.tests.insert(id, test);
            self.get_flow_mut(None).tests.push(id);
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
                .insert("_internal".to_string(), IndexMap::new());
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
            error!("Something has gone wrong, two test invocations have been generated with the same internal ID in flow '{}': \nFirst:\n\n{:?}\n\nSecond:\n\n{:?}", &self.current_flow, &self.tests[&id], &test)
        } else {
            self.test_invocations.insert(id, test);
            self.get_flow_mut(None).test_invocations.push(id);
            Ok(())
        }
    }

    /// Assign the given test to the given invocation, returns an error if neither exists.
    /// Currently, no error will be raised if a test is already assigned to the invocation, it will
    /// be replaced.
    pub fn assign_test_to_inv(&mut self, inv_id: usize, test_id: usize) -> Result<()> {
        if !self.test_invocations.contains_key(&inv_id) {
            return error!(
                "Something has gone wrong, no test invocation exists with ID '{}'",
                inv_id
            );
        }
        if !self.tests.contains_key(&test_id) {
            return error!(
                "Something has gone wrong, no test exists with ID '{}'",
                test_id
            );
        }
        let inv = self.test_invocations.get_mut(&inv_id).unwrap();
        inv.test_id = Some(test_id);
        let test = self.tests.get_mut(&test_id).unwrap();
        test.test_id = Some(inv_id);
        Ok(())
    }

    /// Set the value of the given test attribute.
    /// If the given ID refers to a test invocation then both the invocation and the test will be
    /// checked for a matching attribute.
    /// Currently, if no matching attribute is found then nothing happens.
    /// An error is returned if the test doesn't exist or if the value is the wrong type for the
    /// given parameter.
    /// Calling with value = None will cause all existing settings for the given attribute to be removed.
    pub fn set_test_attr(
        &mut self,
        id: usize,
        name: &str,
        value: Option<ParamValue>,
    ) -> Result<()> {
        if self.test_invocations.contains_key(&id) {
            let inv = self.test_invocations.get_mut(&id).unwrap();
            if inv.has_param(name) {
                inv.set(name, value, true)?;
            } else {
                if let Some(tid) = inv.test_id {
                    if let Some(test) = self.tests.get_mut(&tid) {
                        test.set(name, value, true)?;
                    } else {
                        return error!("Something has gone wrong, no test exists with ID '{}', it was referened by this test invocation: \n{:?}", id, inv);
                    }
                }
            }
            return Ok(());
        }
        if self.tests.contains_key(&id) {
            let test = self.tests.get_mut(&id).unwrap();
            test.set(name, value, true)?;
            return Ok(());
        }
        error!(
            "Something has gone wrong, no test or invocation exists with ID '{}'",
            id
        )
    }
}
