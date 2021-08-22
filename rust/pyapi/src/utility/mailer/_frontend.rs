use crate::application::{get_pyapp, PyApplication};
use crate::utility::metadata::extract_as_metadata;
use crate::utility::results::GenericResult as PyGenericResult;
use origen::core::frontend as ofrontend;
use origen::core::frontend::GenericResult as OGenericResult;
use origen::Metadata;
use origen::Result as OResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashMap;

pub struct Mailer {}

impl ofrontend::Mailer for Mailer {
    fn get_config(&self) -> OResult<HashMap<String, Option<Metadata>>> {
        Ok(self.with_py_mailer(|py, mailer| {
            let r = mailer.call_method0(py, "get_config")?;
            let py_config = r.extract::<&PyDict>(py)?;
            let mut retn = HashMap::new();
            for (k, m) in py_config {
                retn.insert(
                    k.extract::<String>()?,
                    if m.is_none() {
                        None
                    } else {
                        Some(extract_as_metadata(m)?)
                    },
                );
            }
            Ok(retn)
        })?)
    }

    fn send(
        &self,
        _from: &str,
        to: Vec<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        _include_origen_signature: bool,
    ) -> OResult<OGenericResult> {
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
            let pyr = r.extract::<PyRef<PyGenericResult>>(py)?;
            Ok(pyr.into_origen()?)
        })?)
    }

    fn test(&self, to: Option<Vec<&str>>) -> OResult<OGenericResult> {
        Ok(self.with_py_mailer(|py, mailer| {
            let r;
            if let Some(t) = to.as_ref() {
                r = mailer.call_method(py, "test", PyTuple::new(py, [t.to_object(py)]), None)?;
            } else {
                r = mailer.call_method0(py, "test")?;
            }
            let pyr = r.extract::<PyRef<PyGenericResult>>(py)?;
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
