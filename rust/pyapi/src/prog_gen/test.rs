use super::to_param_value;
use origen::prog_gen::{flow_api, ParamValue};
use origen::testers::SupportedTester;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

impl Test {
    pub fn new(
        name: String,
        tester: SupportedTester,
        library_name: String,
        template_name: String,
    ) -> Result<Test> {
        let id = flow_api::define_test(&name, &tester, &library_name, &template_name, None)?;

        Ok(Test {
            name: name,
            tester: tester,
            id: id,
        })
    }

    pub fn set_attr(&self, name: &str, value: ParamValue) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, None)?;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for Test {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}

    //fn __getattr__(&self, _query: &str) -> PyResult<()> {
    //    Ok(())
    //}

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        self.set_attr(name, to_param_value(value)?)?;
        Ok(())
    }
}
