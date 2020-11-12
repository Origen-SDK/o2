use super::V93K;
use crate::prog_gen::{flow_options, to_param_value, Test, TestInvocation};
use origen::prog_gen::ParamValue;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl V93K {
    #[args(kwargs = "**")]
    fn new_test_method(
        &mut self,
        name: String,
        library: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        let t = Test::new(name.clone(), self.tester.to_owned(), library, name)?;
        if let Some(kwargs) = kwargs {
            for (k, v) in kwargs {
                if let Ok(name) = k.extract::<String>() {
                    if !flow_options::is_flow_option(&name) {
                        t.set_attr(&name, to_param_value(v)?)?;
                    }
                } else {
                    return type_error!(&format!(
                        "Illegal test method attribute name type '{}', should be a String",
                        k
                    ));
                }
            }
        }
        Ok(t)
    }

    #[args(kwargs = "**")]
    fn new_test_suite(
        &mut self,
        name: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<TestInvocation> {
        let t = TestInvocation::new(name.clone(), self.tester.to_owned())?;
        t.set_attr("name", ParamValue::String(name.to_owned()))?;
        if let Some(kwargs) = kwargs {
            for (k, v) in kwargs {
                if let Ok(name) = k.extract::<String>() {
                    if !flow_options::is_flow_option(&name) {
                        t.set_attr(&name, to_param_value(v)?)?;
                    }
                } else {
                    return type_error!(&format!(
                        "Illegal test suite attribute name type '{}', should be a String",
                        k
                    ));
                }
            }
        }
        Ok(t)
    }
}
