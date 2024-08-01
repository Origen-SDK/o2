//! Functions for loading test template definitions, e.g. from a json file

use crate::prog_gen::supported_testers::SupportedTester;
use crate::Result;
use phf::phf_map;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

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

lazy_static! {
    static ref LOADED_TESTS: RwLock<HashMap<String, TestTemplate>> = RwLock::new(HashMap::new());
}

/// Test template definitions from json files are read into this structure
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TestTemplate {
    pub parameter_list: Option<HashMap<String, String>>,
    pub aliases: Option<HashMap<String, String>>,
    pub values: Option<HashMap<String, serde_json::Value>>,
    pub parameters: Option<HashMap<String, TestTemplateParameter>>,
    pub class_name: Option<String>,
    pub accepted_values: Option<HashMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
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
    
    let key = format!("{}_{}_{}", &tester_name, lib_name, test_name);
    
    if let Some(t) = LOADED_TESTS.read().unwrap().get(&key) {
        return Ok(t.clone());
    }
    
    let mut available_tests: HashSet<String> = HashSet::new();
    
    // Look for an app-defined library/test first
    for path in crate::PROG_GEN_CONFIG.test_template_load_path() {
        let path = path.join(format!("{}/{}/{}.json", tester_name, lib_name, test_name));
        if path.exists() {
            // Get the file contents
            let contents = std::fs::read_to_string(&path)?;
            let t: TestTemplate = serde_json::from_str(&contents)?;
            LOADED_TESTS.write().unwrap().insert(key, t.clone());
            return Ok(t);
        }
        if path.parent().unwrap().exists() {
            for entry in std::fs::read_dir(path.parent().unwrap())? {
                let entry = entry?;
                if entry.path().is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.ends_with(".json") {
                        available_tests.insert(file_name);
                    }
                }
            }
        }
    }

    match import_test_template(&format!("{}/{}/{}.json", tester_name, lib_name, test_name)) {
        Ok(t) => {
            LOADED_TESTS.write().unwrap().insert(key, t.clone());
            return Ok(t);
        }
        Err(e) => {
            if e.to_string().contains("No test template found at path") {
                log_debug!("{}", e);
                let mut msg = format!(
                    "No test method named '{}' found in library '{}' (for {})",
                    test_name,
                    lib_name,
                    tester
                );
                if available_tests.len() > 0 {
                    let mut available_tests: Vec<String> = available_tests.into_iter().collect();
                    available_tests.sort();
                    msg.push_str("\n\nThe following test methods are available in this library:");
                    for t in available_tests {
                        msg.push_str(&format!("\n  - {}", t));
                    }
                }
                bail!("{}", msg);
            } else {
                bail!("{}", e);
            }
        }
    }
}

fn import_test_template(path: &str) -> Result<TestTemplate> {
    match TEST_TEMPLATES.get(path) {
        None => bail!("No test template found at path '{}'", path),
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
