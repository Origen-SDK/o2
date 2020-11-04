use crate::prog_gen::Test;
//use origen::prog_gen::flow_api;
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

    /// Implements igxl.test_instances.libname.<template_name>
    fn __getattr__(&self, template_name: &str) -> PyResult<Test> {
        let t = Test {
            name: "".to_string(),
            initialized: false, // The test name is not known yet
            tester: self.tester.clone(),
            id: 0, // A placeholder, this will be defined when we know the name
            library_name: self.name.clone(),
            template_name: template_name.to_owned(),
        };
        Ok(t)
    }
}
