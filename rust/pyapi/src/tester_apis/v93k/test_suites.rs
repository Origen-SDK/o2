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
        let t = origen::with_prog_mut(|p| {
            Ok(TestInvocation {
                tester: self.tester.clone(),
                id: v93k_prog::new_test_suite(name, &self.tester, p)?.id,
                test_id: None,
            })
        })?;
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
