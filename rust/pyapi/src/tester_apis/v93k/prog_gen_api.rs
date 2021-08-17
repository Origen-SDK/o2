use super::V93K;
use crate::prog_gen::{Test, TestInvocation};
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
        let t = Test::new(name.clone(), self.tester.to_owned(), library, name, kwargs)?;
        Ok(t)
    }

    #[args(kwargs = "**")]
    fn new_test_suite(
        &mut self,
        name: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<TestInvocation> {
        let t = TestInvocation::new(name.clone(), self.tester.to_owned(), kwargs)?;
        t.set_attr("name", Some(ParamValue::String(name.to_owned())))?;
        Ok(t)
    }
}
