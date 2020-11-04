use crate::prog_gen::TestInvocation;
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
        let mut t = TestInvocation {
            name: name.to_owned(),
            tester: self.tester.clone(),
            id: 0,
            test_id: None,
            test_name: None,
        };
        t.define()?;
        Ok(t)
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
