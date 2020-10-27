use crate::prog_gen::ParamValue;
use crate::testers::SupportedTester;
use crate::Result;
use crate::PROG;

/// Creates a new test instance from the given test instance library template and returns its ID.
/// This function is also responsible for initially defining the given template the
/// first time it is called.
pub fn new_test_instance(lib_name: &str, ti_name: &str, tester: &SupportedTester) -> Result<usize> {
    let prog = PROG.for_tester(tester);

    let tid = match prog.get_test_template_id(lib_name, ti_name) {
        Ok(id) => id,
        Err(_) => {
            load_test_from_lib(tester, lib_name, ti_name)?;
            prog.get_test_template_id(lib_name, ti_name)?
        }
    };
    let id = prog.create_duplicate_test(tid)?;
    prog.with_test_mut(id, |t| {
        t.indirect = false;
        Ok(())
    })?;
    Ok(id)
}

/// Loads a test method definition e.g. from a json file
fn load_test_from_lib(tester: &SupportedTester, lib_name: &str, test_name: &str) -> Result<()> {
    log_trace!(
        "Looking for test method definition '{}' in library '{}' (for {})",
        test_name,
        lib_name,
        tester
    );
    let prog = PROG.for_tester(tester);
    prog.create_test_library(lib_name);
    match tester {
        SupportedTester::ULTRAFLEX => {
            let _ = prog.create_test_template(lib_name, test_name, |t| {
                let base_template =
                    super::super::import_test_template("teradyne/ultraflex/test_instance.json")?;
                t.import_test_template(&base_template);

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
                t.import_test_template(&template)
            })?;
        }
        _ => return error!("No libraries exist for tester '{}'", tester),
    }

    Ok(())
}
