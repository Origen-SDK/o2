use crate::prog_gen::ParamValue;
use crate::testers::SupportedTester;
use crate::Result;
use crate::PROG;

/// Creates a new test suite with the given name and returns its ID.
/// This function is also responsible for initially defining a test suite template the
/// first time it is called.
pub fn new_test_suite(name: &str, tester: &SupportedTester) -> Result<usize> {
    let prog = PROG.for_tester(tester);

    let tid = match prog.get_test_template_id("_internal", "test_suite") {
        Ok(id) => id,
        Err(_) => {
            create_internal_test_lib(tester)?;
            prog.get_test_template_id("_internal", "test_suite")
                .unwrap()
        }
    };
    let id = prog.create_duplicate_test(tid)?;
    prog.with_test_mut(id, |t| {
        t.name = name.to_owned();
        t.indirect = false;
        t.set("name", ParamValue::String(name.to_owned()))?;
        Ok(())
    })?;
    Ok(id)
}

/// Creates a new test method instance from the given test method library template and returns its ID.
/// This function is also responsible for initially defining the given template the
/// first time it is called.
pub fn new_test_method(lib_name: &str, tm_name: &str, tester: &SupportedTester) -> Result<usize> {
    let prog = PROG.for_tester(tester);

    let tid = match prog.get_test_template_id(lib_name, tm_name) {
        Ok(id) => id,
        Err(_) => {
            load_test_from_lib(tester, lib_name, tm_name)?;
            prog.get_test_template_id(lib_name, tm_name)?
        }
    };
    let id = prog.create_duplicate_test(tid)?;
    prog.with_test_mut(id, |t| {
        t.indirect = false;
        Ok(())
    })?;
    Ok(id)
}

/// Creates the templates for test suites and test methods
fn create_internal_test_lib(tester: &SupportedTester) -> Result<()> {
    let prog = PROG.for_tester(tester);

    prog.create_test_library("_internal");

    let _ = prog.create_test_template("_internal", "test_suite", |t| {
        let template =
            super::super::import_test_template("advantest/smt7/_internal/test_suite.json")?;
        t.import_test_template(&template)
    })?;
    Ok(())
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
        SupportedTester::V93KSMT7 => {
            let _ = prog.create_test_template(lib_name, test_name, |t| {
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
                t.import_test_template(&template)
            })?;
        }
        _ => return error!("No libraries exist for tester '{}'", tester),
    }

    Ok(())
}
