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
