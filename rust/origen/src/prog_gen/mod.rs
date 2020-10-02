mod advantest;
mod teradyne;
mod test_program;
mod tests;

use phf::phf_map;
use std::collections::HashMap;
pub use test_program::TestProgram;

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

#[derive(Debug, Deserialize)]
struct TestLibrary {
    class_name_prefix: Option<String>,
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_all_templates_import_cleanly() {
        for (file, content) in TEST_TEMPLATES.entries() {
            if file.ends_with("library.json") {
                let _: TestLibrary =
                    serde_json::from_str(content).expect(&format!("Failed to import {}", file));
            } else {
                let _: TestTemplate =
                    serde_json::from_str(content).expect(&format!("Failed to import {}", file));
            }
        }
    }
}
