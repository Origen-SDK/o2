use crate::application::{get_pyapp, PyApplication};
use origen::core::frontend as ofrontend;
use origen::Result as OResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyapi_metal::prelude::typed_value;
use pyapi_metal::prelude::{om, PyOutcome};

pub struct Mailer {}

impl ofrontend::Mailer for Mailer {
    fn get_config(&self) -> OResult<typed_value::TypedValueMap> {
        Ok(self.with_py_mailer(|py, mailer| {
            let r = mailer.call_method0(py, "get_config")?;
            let py_config = r.extract::<&PyDict>(py)?;
            Ok(typed_value::from_pydict(py_config)?)
        })?)
    }

    fn send(
        &self,
        _from: &str,
        to: Vec<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        _include_origen_signature: bool,
    ) -> OResult<om::Outcome> {
        Ok(self.with_py_mailer(|py, mailer| {
            let r = mailer.call_method(
                py,
                "send",
                PyTuple::new(
                    py,
                    [to.to_object(py), body.to_object(py), subject.to_object(py)],
                ),
                None,
            )?;
            let pyr = r.extract::<PyRef<PyOutcome>>(py)?;
            Ok(pyr.into_origen()?)
        })?)
    }

    fn test(&self, to: Option<Vec<&str>>) -> OResult<om::Outcome> {
        Ok(self.with_py_mailer(|py, mailer| {
            let r;
            if let Some(t) = to.as_ref() {
                r = mailer.call_method(py, "test", PyTuple::new(py, [t.to_object(py)]), None)?;
            } else {
                r = mailer.call_method0(py, "test")?;
            }
            let pyr = r.extract::<PyRef<PyOutcome>>(py)?;
            Ok(pyr.into_origen()?)
        })?)
    }
}

impl Mailer {
    fn with_py_mailer<T, F>(&self, mut func: F) -> PyResult<T>
    where
        F: FnMut(Python, PyObject) -> PyResult<T>,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rs = PyApplication::_get_mailer(pyapp, py)?;
        func(py, rs)
    }
}
