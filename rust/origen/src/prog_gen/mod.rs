pub mod advantest;
mod model;
pub mod teradyne;

use crate::generator::ast::Node;
use crate::generator::ast::AST;
use crate::testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
pub use model::ParamType;
pub use model::ParamValue;
pub use model::Test;
use model::TestProgram;
use phf::phf_map;
use std::collections::HashMap;
use std::sync::RwLock;

// This includes a map of all test template files, it is built by build.rs at compile time.
// All files in each sub-directory of prog_gen/test_templates are accessible via a map with the
// following structure:
//
//      TEST_TEMPLATES = {
//        "advantest/smt7/dc_tml/continuity.json" => "...",
//        "advantest/smt7/dc_tml/dps_connectivity.json" => "...",
//        ...
//        "advantest/smt7/ac_tml/..." => "...",
//        ...
//        "teradyne/j750/std/apmu_powersupply.json" => "...",
//        "teradyne/j750/std/board_pmu.json" => "...",
//      }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new build.
include!(concat!(env!("OUT_DIR"), "/test_templates.rs"));

/// The TestPrograms is a singleton which lives for the entire duration of an Origen program
/// generation run (the whole execution of an 'origen g' command), it is instantiated as
/// origen::PROG.
/// It provides long term storage for the test program model, similar to how the DUT provides long
/// term storage of the regs and other DUT models.
/// A complete test program model is maintained for each tester target, each model stores the test
/// templates and test instances for the given tester.
/// A common flow AST is shared by all testers and is therefore stored in this central data struct.
#[derive(Debug, Default)]
pub struct TestPrograms {
    /// A number of empty models are created initially and then mapped to a tester via the
    /// assignments hash.
    /// It is done this way so that the models don't need to be put behind a lock to make it
    /// easy to get a reference on them.
    models: Vec<TestProgram>,
    /// Assignments map a specific tester type to a model in the models vector
    assignments: RwLock<HashMap<SupportedTester, usize>>,
    /// This keeps track of what testers are selected by 'with specific tester' blocks in the application.
    /// It is implemented as a stack, allowing the application to select multiple testers at a time and then
    /// optionally select a subset via a nested with block.
    current_testers: RwLock<Vec<Vec<SupportedTester>>>,
    /// Flows are represented as an AST, which will contain references to tests from the
    /// above collection, the last flow is the current one and so an IndexMap (ordered) instead of
    /// regular HashMap.
    flows: RwLock<IndexMap<String, AST>>,
}

impl TestPrograms {
    pub fn new() -> Self {
        let mut models: Vec<TestProgram> = vec![];
        for _ in 0..20 {
            // Assumes an application will never be targetting more than 20 testers at once!
            models.push(TestProgram::new());
        }
        Self {
            models: models,
            assignments: RwLock::new(HashMap::new()),
            current_testers: RwLock::new(vec![]),
            flows: RwLock::new(IndexMap::new()),
        }
    }

    /// Returns the test program model for the given tester
    pub fn for_tester(&self, tester: &SupportedTester) -> &TestProgram {
        {
            if let Some(x) = self.assignments.read().unwrap().get(tester) {
                return &self.models[x.to_owned()];
            }
        }
        &self.models[self.assign_tester(tester)]
    }

    fn assign_tester(&self, tester: &SupportedTester) -> usize {
        let mut assignments = self.assignments.write().unwrap();
        let id = assignments.len();
        assignments.insert(tester.to_owned(), id);
        self.models[id].set_tester(tester);
        id
    }

    pub fn push_current_testers(&self, testers: Vec<SupportedTester>) -> Result<usize> {
        if testers.is_empty() {
            return error!("No tester type(s) given");
        }
        let mut current_testers = self.current_testers.write().unwrap();
        // When some testers are already selected, the application is only allowed to select a subset of them,
        // so verify that all given testers are already contained in the last selection
        if !current_testers.is_empty() {
            let last = current_testers.last().unwrap();
            for t in &testers {
                if !last.contains(t) {
                    return error!(
                        "Can't select tester '{}' within a block that already selects '{:?}'",
                        t, last
                    );
                }
            }
        }
        current_testers.push(testers.clone());
        let n = node!(TesterSpecific, testers);
        self.push_and_open(n)
    }

    pub fn pop_current_testers(&self, ref_id: usize) -> Result<()> {
        let mut current_testers = self.current_testers.write().unwrap();
        if current_testers.is_empty() {
            return error!("There has been an attempt to close a tester-specific block, but none is currently open in the program generator");
        }
        let _ = current_testers.pop();
        self.close(ref_id)?;
        Ok(())
    }

    /// Push a new terminal node into the AST for the current flow
    pub fn push(&self, node: Node) -> Result<()> {
        self.with_current_flow_mut(|t| {
            t.push(node);
            Ok(())
        })
    }

    /// Append multiple nodes to the AST for the current flow
    pub fn append(&self, nodes: &mut Vec<Node>) -> Result<()> {
        self.with_current_flow_mut(|t| {
            t.append(nodes);
            Ok(())
        })
    }

    /// Push a new node into the current flow and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// A reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&self, node: Node) -> Result<usize> {
        self.with_current_flow_mut(|t| Ok(t.push_and_open(node)))
    }

    /// Close the currently open node in the current test flow
    pub fn close(&self, ref_id: usize) -> Result<()> {
        self.with_current_flow_mut(|t| t.close(ref_id))
    }

    /// Replace the node n - offset with the given node, use offset = 0 to
    /// replace the last node that was pushed.
    /// Fails if the AST has no children yet or if the offset is otherwise out
    /// of range.
    pub fn replace(&self, node: Node, offset: usize) -> Result<()> {
        self.with_current_flow_mut(|t| t.replace(node, offset))
    }

    /// Returns a copy of node n - offset, an offset of 0 means
    /// the last node pushed.
    /// Fails if the offset is out of range.
    pub fn get(&self, offset: usize) -> Result<Node> {
        self.with_current_flow(|t| t.get(offset))
    }

    /// Start a new flow, will fail if a flow with the given name already exists.
    /// Called at the start of a top-level flow.
    pub fn start_flow(&self, name: &str) -> Result<usize> {
        let mut flows = self.flows.write().unwrap();
        if flows.contains_key(name) {
            return error!("A flow already exists called '{}'", name);
        }
        let mut ast = AST::new();
        let ref_id = ast.push_and_open(node!(PGMFlow, name.to_owned()));
        flows.insert(name.to_owned(), ast);
        Ok(ref_id)
    }

    /// Called at the end of a top-level flow
    pub fn end_flow(&self, _ref_id: usize) -> Result<()> {
        Ok(())
    }

    /// Called at the start of a sub-flow
    pub fn start_sub_flow(&self, name: &str) -> Result<usize> {
        let n = node!(PGMSubFlow, name.to_owned());
        self.push_and_open(n)
    }

    /// Called at the end of a sub-flow
    pub fn end_sub_flow(&self, ref_id: usize) -> Result<()> {
        self.close(ref_id)
    }

    /// Execute the given function with a reference to an index map containing all flows, the map
    /// will simply be empty if there are no flows.
    /// The result of the given function is returned.
    pub fn with_all_flows<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&IndexMap<String, AST>) -> Result<T>,
    {
        let flows = self.flows.read().unwrap();
        func(&flows)
    }

    /// Execute the given function with a reference to the current flow.
    /// Returns an error if there is no current flow, otherwise the result of the given function.
    pub fn with_current_flow<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&AST) -> Result<T>,
    {
        let flows = self.flows.read().unwrap();
        match flows.values().last() {
            None => error!("Something has gone wrong, a reference has been made to the current flow when there is none"),
            Some(f) => func(f),
        }
    }

    /// Execute the given function with a mutable reference to the current flow.
    /// Returns an error if there is no current flow, otherwise the result of the given function.
    pub fn with_current_flow_mut<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&mut AST) -> Result<T>,
    {
        let mut flows = self.flows.write().unwrap();
        match flows.values_mut().last() {
            None => error!("Something has gone wrong, a reference has been made to the current flow when there is none"),
            Some(f) => func(f),
        }
    }

    /// Add the given test to the current flow
    pub fn add_test(
        &self,
        name: String,
        tester: Option<SupportedTester>,
        test_id: Option<usize>,
        invocation_id: Option<usize>,
    ) -> Result<()> {
        let n = node!(PGMTest, name, tester, test_id, invocation_id);
        self.push(n)
    }
}
/// Test template definitions from json files are read into this structure
#[derive(Debug, Deserialize)]
pub struct TestTemplate {
    parameter_list: Option<HashMap<String, String>>,
    aliases: Option<HashMap<String, String>>,
    values: Option<HashMap<String, serde_json::Value>>,
    parameters: Option<HashMap<String, TestTemplateParameter>>,
    class_name: Option<String>,
    accepted_values: Option<HashMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
pub struct TestTemplateParameter {
    #[serde(rename(deserialize = "type"))]
    kind: Option<String>,
    aliases: Option<Vec<String>>,
    value: Option<serde_json::Value>,
    accepted_values: Option<Vec<serde_json::Value>>,
}

pub fn import_test_template(path: &str) -> Result<TestTemplate> {
    match TEST_TEMPLATES.get(path) {
        None => return error!("No test template found at path '{}'", path),
        Some(s) => Ok(serde_json::from_str(s)?),
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_all_templates_import_cleanly() {
        for (file, content) in TEST_TEMPLATES.entries() {
            let _: TestTemplate =
                serde_json::from_str(content).expect(&format!("Failed to import {}", file));
        }
    }
}
