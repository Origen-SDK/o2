use crate::prog_gen::Test;
use origen::prog_gen::teradyne::common as uflex_prog;
use origen::testers::SupportedTester;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInstances {
    pub tester: SupportedTester,
}

#[pyproto]
impl PyObjectProtocol for TestInstances {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<TIL> {
        Ok(TIL {
            tester: self.tester.clone(),
            name: query.to_string(),
        })
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TIL {
    tester: SupportedTester,
    name: String,
}

#[pyproto]
impl PyObjectProtocol for TIL {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<Test> {
        let t = origen::with_prog_mut(|p| {
            Ok(Test {
                name: query.to_string(),
                id: uflex_prog::new_test_instance(&self.name, query, &self.tester, p)?.id,
                tester: self.tester.clone(),
                initialized: false,
            })
        })?;
        Ok(t)
    }
}
