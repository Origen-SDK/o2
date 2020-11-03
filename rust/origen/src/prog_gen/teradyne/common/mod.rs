use crate::prog_gen::{Database, Test};
use crate::testers::SupportedTester;
use crate::Result;

/// Creates a new test instance from the given test instance library template and return a mutable
/// reference to it.
/// This function is also responsible for initially defining the given template the
/// first time it is called.
pub fn new_test_instance<'a>(
    lib_name: &str,
    ti_name: &str,
    tester: &SupportedTester,
    database: &'a mut Database,
) -> Result<&'a mut Test> {
    let tid = match database.get_test_template_id(tester, lib_name, ti_name) {
        Ok(id) => id,
        Err(_) => {
            load_test_from_lib(tester, lib_name, ti_name, database)?;
            database.get_test_template_id(tester, lib_name, ti_name)?
        }
    };
    let t = database.create_duplicate_test(tid)?;
    t.indirect = false;
    Ok(t)
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
        SupportedTester::ULTRAFLEX => {
            let t = database.create_test_template(tester, lib_name, test_name)?;
            let base_template =
                super::super::import_test_template("teradyne/ultraflex/test_instance.json")?;
            t.import_test_template(&base_template)?;

            let template = match super::super::import_test_template(&format!(
                "teradyne/ultraflex/{}/{}.json",
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
