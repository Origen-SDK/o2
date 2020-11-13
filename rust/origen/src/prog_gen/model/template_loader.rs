//! Functions for loading test template definitions, e.g. from a json file

use crate::testers::SupportedTester;
use crate::Result;
use phf::phf_map;
use std::collections::HashMap;

// This includes a map of all test template files, it is built by build.rs at compile time.
// All files in each sub-directory of prog_gen/test_templates are accessible via a map with the
// following structure:
//
//      TEST_TEMPLATES = {
//        "v93ksmt7/dc_tml/continuity.json" => "...",
//        "v93ksmt7/dc_tml/dps_connectivity.json" => "...",
//        ...
//        "v93ksmt7/ac_tml/..." => "...",
//        ...
//        "j750/std/apmu_powersupply.json" => "...",
//        "j750/std/board_pmu.json" => "...",
//      }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new build.
include!(concat!(env!("OUT_DIR"), "/test_templates.rs"));

/// Test template definitions from json files are read into this structure
#[derive(Debug, Deserialize)]
pub struct TestTemplate {
    pub parameter_list: Option<HashMap<String, String>>,
    pub aliases: Option<HashMap<String, String>>,
    pub values: Option<HashMap<String, serde_json::Value>>,
    pub parameters: Option<HashMap<String, TestTemplateParameter>>,
    pub class_name: Option<String>,
    pub accepted_values: Option<HashMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
pub struct TestTemplateParameter {
    #[serde(rename(deserialize = "type"))]
    pub kind: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub value: Option<serde_json::Value>,
    pub accepted_values: Option<Vec<serde_json::Value>>,
}

/// Loads a test method definition e.g. from a json file into the returned TestTemplate,
/// returns an error if a corresponding definition file cannot be found
pub fn load_test_from_lib(
    tester: &SupportedTester,
    lib_name: &str,
    test_name: &str,
) -> Result<TestTemplate> {
    log_trace!(
        "Looking for test template definition '{}' in library '{}' (for {})",
        test_name,
        lib_name,
        tester
    );
    let tester_name = tester.to_string().to_lowercase();

    // TODO: look for templates in an app-defined load path here, see:
    // https://github.com/Origen-SDK/o2/pull/126#issuecomment-717939430

    match import_test_template(&format!("{}/{}/{}.json", tester_name, lib_name, test_name)) {
        Ok(t) => return Ok(t),
        Err(e) => {
            if e.to_string().contains("No test template found at path") {
                log_debug!("{}", e);
                return error!(
                    "No test method named '{}' found in library '{}' (for {})",
                    test_name, lib_name, tester
                );
            } else {
                return error!("{}", e);
            }
        }
    }
}

fn import_test_template(path: &str) -> Result<TestTemplate> {
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
