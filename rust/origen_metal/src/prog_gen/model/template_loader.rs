//! Functions for loading test template definitions, e.g. from a json file

use crate::prog_gen::supported_testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
use phf::phf_map;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
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
    /// The templates loaded so far, with the template load path that each was found from. Which
    /// file a template comes from, an app-defined one or a built-in one, depends on the load path,
    /// so a template is only served from here while the load path is the same.
    static ref LOADED_TESTS: RwLock<HashMap<String, (Vec<PathBuf>, TestTemplate)>> =
        RwLock::new(HashMap::new());
}

/// Test template definitions from json files are read into this structure
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TestTemplate {
    pub parameter_list: Option<IndexMap<String, String>>,
    pub aliases: Option<IndexMap<String, String>>,
    pub values: Option<IndexMap<String, serde_json::Value>>,
    pub parameters: Option<IndexMap<String, TestTemplateParameter>>,
    pub collections: Option<IndexMap<String, TestTemplateCollection>>,
    pub class_name: Option<String>,
    pub accepted_values: Option<IndexMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TestTemplateCollection {
    pub parameters: Option<IndexMap<String, TestTemplateParameter>>,
    pub collections: Option<IndexMap<String, TestTemplateCollection>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TestTemplateParameter {
    #[serde(alias = "type")]
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

    let load_path = crate::PROG_GEN_CONFIG.test_template_load_path();

    if let Some((loaded_from, t)) = LOADED_TESTS.read().unwrap().get(&key) {
        if loaded_from == &load_path {
            return Ok(t.clone());
        }
    }

    let mut available_tests: HashSet<String> = HashSet::new();

    // Look for an app-defined library/test first
    for path in &load_path {
        let path = path.join(format!("{}/{}/{}.json", tester_name, lib_name, test_name));
        if path.exists() {
            // Get the file contents
            let contents = std::fs::read_to_string(&path)?;
            let t: TestTemplate = serde_json::from_str(&contents)?;
            LOADED_TESTS
                .write()
                .unwrap()
                .insert(key, (load_path.clone(), t.clone()));
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
            LOADED_TESTS
                .write()
                .unwrap()
                .insert(key, (load_path.clone(), t.clone()));
            return Ok(t);
        }
        Err(e) => {
            if e.to_string().contains("No test template found at path") {
                log_debug!("{}", e);
                let mut msg = format!(
                    "No test method named '{}' found in library '{}' (for {})",
                    test_name, lib_name, tester
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

    fn write_template(dir: &std::path::Path, file: &str, json: &str) {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, json).unwrap();
    }

    fn parameters(template: &TestTemplate) -> Vec<String> {
        template
            .parameters
            .as_ref()
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default()
    }

    #[test]
    fn a_loaded_template_is_not_served_once_the_load_path_changes() {
        let original_load_path = crate::PROG_GEN_CONFIG.test_template_load_path();
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let file = "v93ksmt8/load_path_cache_lib/T.json";
        write_template(
            a.path(),
            file,
            r#"{"parameters": {"x": {"kind": "string"}}}"#,
        );
        write_template(
            b.path(),
            file,
            r#"{"parameters": {"x": {"kind": "string"}, "y": {"kind": "string"}}}"#,
        );
        // A built-in template, overridden by an app-defined one in b
        // One which no other test loads, as the load path is global
        let builtin = "v93ksmt7/dc_tml/dps_status.json";
        assert!(TEST_TEMPLATES.get(builtin).is_some());
        write_template(
            b.path(),
            builtin,
            r#"{"class_name": "load_path_cache.Override"}"#,
        );
        let load = |tester: &SupportedTester, lib: &str, test: &str| {
            load_test_from_lib(tester, lib, test).unwrap()
        };

        let result = std::panic::catch_unwind(|| {
            crate::PROG_GEN_CONFIG.set_test_template_load_path(vec![a.path().to_path_buf()]);
            let t = load(&SupportedTester::V93KSMT8, "load_path_cache_lib", "T");
            assert_eq!(parameters(&t), vec!["x"]);
            let builtin = load(&SupportedTester::V93KSMT7, "dc_tml", "dps_status");
            assert_ne!(
                builtin.class_name.as_deref(),
                Some("load_path_cache.Override")
            );

            crate::PROG_GEN_CONFIG.set_test_template_load_path(vec![b.path().to_path_buf()]);
            let t = load(&SupportedTester::V93KSMT8, "load_path_cache_lib", "T");
            assert_eq!(parameters(&t), vec!["x", "y"]);
            let overridden = load(&SupportedTester::V93KSMT7, "dc_tml", "dps_status");
            assert_eq!(
                overridden.class_name.as_deref(),
                Some("load_path_cache.Override")
            );

            // And back again
            crate::PROG_GEN_CONFIG.set_test_template_load_path(vec![a.path().to_path_buf()]);
            let t = load(&SupportedTester::V93KSMT8, "load_path_cache_lib", "T");
            assert_eq!(parameters(&t), vec!["x"]);
            let builtin = load(&SupportedTester::V93KSMT7, "dc_tml", "dps_status");
            assert_ne!(
                builtin.class_name.as_deref(),
                Some("load_path_cache.Override")
            );
        });
        crate::PROG_GEN_CONFIG.set_test_template_load_path(original_load_path);
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    #[test]
    fn test_all_templates_import_cleanly() {
        for (file, content) in TEST_TEMPLATES.entries() {
            let _: TestTemplate =
                serde_json::from_str(content).expect(&format!("Failed to import {}", file));
        }
    }

    #[test]
    fn test_kind_alias_and_recursive_collections_import() {
        let template: TestTemplate = serde_json::from_str(
            r#"{
                "parameters": {
                    "enableSoftset": {
                        "kind": "boolean",
                        "value": "false"
                    }
                },
                "collections": {
                    "tsen": {
                        "parameters": {
                            "zDataDeltaLimit": {
                                "kind": "double",
                                "value": "0.0"
                            }
                        },
                        "collections": {
                            "registers": {
                                "parameters": {
                                    "zLowLimit": {
                                        "type": "double",
                                        "value": "-Infinity"
                                    }
                                }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            template.parameters.as_ref().unwrap()["enableSoftset"]
                .kind
                .as_deref(),
            Some("boolean")
        );
        assert!(template.collections.as_ref().unwrap().contains_key("tsen"));
        assert!(template.collections.as_ref().unwrap()["tsen"]
            .collections
            .as_ref()
            .unwrap()
            .contains_key("registers"));
        assert_eq!(
            template.collections.as_ref().unwrap()["tsen"]
                .collections
                .as_ref()
                .unwrap()["registers"]
                .parameters
                .as_ref()
                .unwrap()["zLowLimit"]
                .kind
                .as_deref(),
            Some("double")
        );
    }
}
