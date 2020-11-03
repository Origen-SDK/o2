use crate::prog_gen::ParamValue;
use crate::prog_gen::{Database, Test};
use crate::testers::SupportedTester;
use crate::Result;

/// Creates a new test suite with the given name and returns a mutable reference to it.
/// This function is also responsible for initially defining a test suite template the
/// first time it is called.
pub fn new_test_suite<'a>(
    name: &str,
    tester: &SupportedTester,
    database: &'a mut Database,
) -> Result<&'a mut Test> {
    let tid = match database.get_test_template_id(tester, "_internal", "test_suite") {
        Ok(id) => id,
        Err(_) => {
            create_internal_test_lib(tester, database)?;
            database
                .get_test_template_id(tester, "_internal", "test_suite")
                .unwrap()
        }
    };
    let mut t = database.create_duplicate_test(tid)?;
    t.name = name.to_owned();
    t.indirect = false;
    t.set("name", ParamValue::String(name.to_owned()))?;
    Ok(t)
}

/// Creates a new test method instance from the given test method library template and returns a mutable
/// reference to it.
/// This function is also responsible for initially defining the given template the
/// first time it is called.
pub fn new_test_method<'a>(
    lib_name: &str,
    tm_name: &str,
    tester: &SupportedTester,
    database: &'a mut Database,
) -> Result<&'a mut Test> {
    let tid = match database.get_test_template_id(tester, lib_name, tm_name) {
        Ok(id) => id,
        Err(_) => {
            load_test_from_lib(tester, lib_name, tm_name, database)?;
            database.get_test_template_id(tester, lib_name, tm_name)?
        }
    };
    let mut t = database.create_duplicate_test(tid)?;
    t.indirect = false;
    Ok(t)
}

/// Creates the templates for test suites and test methods
fn create_internal_test_lib(tester: &SupportedTester, database: &mut Database) -> Result<()> {
    database.create_test_library(tester, "_internal");

    let t = database.create_test_template(tester, "_internal", "test_suite")?;
    let template = super::super::import_test_template("advantest/smt7/_internal/test_suite.json")?;
    t.import_test_template(&template)?;
    Ok(())
}

/// Loads a test method definition e.g. from a json file
fn load_test_from_lib(
    tester: &SupportedTester,
    lib_name: &str,
    test_name: &str,
    database: &mut Database,
) -> Result<()> {
    log_trace!(
        "Looking for test method definition '{}' in library '{}' (for {})",
        test_name,
        lib_name,
        tester
    );
    database.create_test_library(tester, lib_name);
    match tester {
        SupportedTester::V93KSMT7 => {
            let t = database.create_test_template(tester, lib_name, test_name)?;
            let template = match super::super::import_test_template(&format!(
                "advantest/smt7/{}/{}.json",
                lib_name, test_name
            )) {
                Ok(t) => t,
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
            };
            t.import_test_template(&template)?;
        }
        _ => return error!("No libraries exist for tester '{}'", tester),
    }

    Ok(())
}
