pub mod advantest;
pub mod flow_api;
mod flow_manager;
mod model;
mod processors;
pub mod teradyne;

use crate::Result;
pub use flow_manager::FlowManager;
pub use model::BinType;
pub use model::Database;
pub use model::FlowCondition;
pub use model::FlowID;
pub use model::GroupType;
pub use model::ParamType;
pub use model::ParamValue;
pub use model::PatternGroupType;
pub use model::Test;
use phf::phf_map;
use std::collections::HashMap;

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
