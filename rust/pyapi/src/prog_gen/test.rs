use super::to_param_value;
use origen::prog_gen::{flow_api, ParamValue};
use origen::testers::SupportedTester;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions::TypeError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub tester: SupportedTester,
    pub initialized: bool,
    pub id: usize,
    pub library_name: String,
    pub template_name: String,
}

#[pymethods]
impl Test {
    // This implements ..test_instances.std.functional(<name>)
    #[__call__]
    fn __call__(&mut self, name: &str) -> PyResult<Test> {
        if self.initialized {
            return Err(TypeError::py_err(
                "'Test' object is not callable".to_string(),
            ));
        }
        self.initialized = true;
        self.name = name.to_owned();
        self.define()?;
        flow_api::set_test_attr(
            self.id,
            "test_name",
            ParamValue::String(name.to_owned()),
            None,
        )?;
        Ok(self.clone())
    }
}

impl Test {
    pub fn define(&mut self) -> Result<()> {
        self.id = flow_api::define_test(
            &self.name,
            &self.tester,
            &self.library_name,
            &self.template_name,
            None,
        )?;
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
        flow_api::set_test_attr(self.id, name, to_param_value(value)?, None)?;
        Ok(())
    }
}
