use crate::prog_gen::Test;
use origen::prog_gen::advantest::common as v93k_prog;
use origen::testers::SupportedTester;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestMethods {
    pub tester: SupportedTester,
}

#[pyproto]
impl PyObjectProtocol for TestMethods {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<TML> {
        Ok(TML {
            tester: self.tester.clone(),
            name: query.to_string(),
        })
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TML {
    tester: SupportedTester,
    name: String,
}

#[pyproto]
impl PyObjectProtocol for TML {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<Test> {
        Ok(Test {
            name: query.to_string(),
            id: v93k_prog::new_test_method(&self.name, query, &self.tester)?,
            tester: self.tester.clone(),
            initialized: true,
        })
    }
}
