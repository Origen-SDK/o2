use crate::prog_gen::Test;
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
    fn __getattr__(&self, test_method_name: &str) -> PyResult<Test> {
        let mut t = Test {
            name: test_method_name.to_string(),
            initialized: true,
            tester: self.tester.clone(),
            id: 0,
            library_name: self.name.clone(),
            template_name: test_method_name.to_owned(),
        };
        t.define()?;
        Ok(t)
    }
}
