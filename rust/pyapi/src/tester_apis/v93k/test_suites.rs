use crate::prog_gen::TestInvocation;
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
    pub fn add(&self, name: &str) -> PyResult<TestInvocation> {
        Ok(TestInvocation {
            tester: self.tester.clone(),
            id: v93k_prog::new_test_suite(name, &self.tester)?,
            test_id: None,
        })
    }
}

//#[pymethods]
//impl TestSuite {
//    #[setter]
//    pub fn test_method(&mut self, tm: Test) -> PyResult<()> {
//        self.test_method_id = Some(tm.id);
//        Ok(())
//    }
//}
