mod advantest;
mod model;
mod teradyne;

use crate::testers::SupportedTester;
use crate::Result;
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
//        "teradyne/j750/apmu_powersupply.json" => "...",
//        "teradyne/j750/board_pmu.json" => "...",
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
/// A complete test program model is maintained for each tester target.
#[derive(Debug, Default)]
pub struct TestPrograms {
    /// A number of empty models are created initially and then mapped to a tester via the
    /// assignments hash.
    /// It is done this way so that the models don't need to be put behind a lock to make it
    /// easy to get a reference on them.
    models: Vec<TestProgram>,
    assignments: RwLock<HashMap<SupportedTester, usize>>,
    /// This keeps track of what testers are selected by 'with specific tester' blocks in the application.
    /// It is implemented as a stack, allowing the application to select multiple testers at a time and then
    /// optionally select a subset via a nested with block.
    current_testers: RwLock<Vec<Vec<SupportedTester>>>,
}

impl TestPrograms {
    pub fn new() -> Self {
        let mut models: Vec<TestProgram> = vec![];
        for i in 0..20 {
            // Assumes an application will never be targetting more than 20 testers at once!
            models.push(TestProgram::new());
        }
        Self {
            models: models,
            assignments: RwLock::new(HashMap::new()),
            current_testers: RwLock::new(vec![]),
        }
    }

    /// Returns the test program model for the given tester
    pub fn for_tester(&self, tester: &SupportedTester) -> &TestProgram {
        match self.assignments.read().unwrap().get(tester) {
            Some(x) => &self.models[x.to_owned()],
            None => &self.models[self.assign_tester(tester)],
        }
    }

    fn assign_tester(&self, tester: &SupportedTester) -> usize {
        let mut assignments = self.assignments.write().unwrap();
        let id = assignments.len();
        assignments.insert(tester.to_owned(), id);
        self.models[id].set_tester(tester);
        id
    }

    pub fn push_current_testers(&self, testers: Vec<SupportedTester>) -> Result<()> {
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
        current_testers.push(testers);
        Ok(())
    }

    ///// Returns the current tester, will error if more than one tester is currently selected
    //fn current_tester(&self) -> Result<SupportedTester> {
    //    let t = *self.current_testers.read().unwrap();
    //    if t.len() != 1 {
    //        if t.len() == 0 {
    //            error!("No tester is currently selected by the test program")
    //        } else {
    //            error!("Expected only one tester to be selected, but the following were selected: {:?}", &t)
    //        }
    //    } else {
    //        Ok(t[0].clone())
    //    }
    //}

    ///// Execute the given function with the test program selecting the given tester
    ///// types.
    ///// At the end the test program's tester selection will be restored to it's original
    ///// value.
    //pub fn for_current_testers<T, F>(&self, testers: Vec<&SupportedTester>, mut func: F) -> Result<T>
    //where
    //    F: FnMut(&TestProgram) -> Result<T>,
    //{
    //    let mut current_testers = *self.current_testers.write().unwrap();
    //    let orig: Vec<SupportedTester> = current_testers.drain(..).collect();
    //    for t in testers {
    //        current_testers.push(t.to_owned());
    //    }
    //    let tp = self.
    //    let result = func(&self);
    //    current_testers.clear();
    //    for t in orig.drain(..) {
    //        current_testers.push(t);
    //    }
    //    result
    //}
}

/// Test template definitions from json files are read into this structure
#[derive(Debug, Deserialize)]
struct TestTemplate {
    parameter_list: Option<HashMap<String, String>>,
    aliases: Option<HashMap<String, String>>,
    values: Option<HashMap<String, serde_json::Value>>,
    parameters: Option<HashMap<String, TestTemplateParameter>>,
    class_name: Option<String>,
    accepted_values: Option<HashMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
struct TestTemplateParameter {
    kind: Option<String>,
    aliases: Option<Vec<String>>,
    value: Option<serde_json::Value>,
    accepted_values: Option<Vec<serde_json::Value>>,
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
