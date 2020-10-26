use crate::prog_gen::Test;
use origen::prog_gen::advantest::common as v93k_prog;
use origen::testers::SupportedTester;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestSuites {
    pub tester: SupportedTester,
}

#[pymethods]
impl TestSuites {
    pub fn add(&self, name: &str) -> PyResult<TestSuite> {
        Ok(TestSuite {
            name: name.to_string(),
            tester: self.tester.clone(),
            test_suite_id: v93k_prog::new_test_suite(name, &self.tester)?,
            test_method_id: None,
        })
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tester: SupportedTester,
    test_suite_id: usize,
    test_method_id: Option<usize>,
}

#[pymethods]
impl TestSuite {
    #[setter]
    pub fn test_method(&mut self, tm: Test) -> PyResult<()> {
        self.test_method_id = Some(tm.id);
        Ok(())
    }
}
