use super::{with_frontend_mod, PY_FRONTEND};
use crate::{bail_with_runtime_error, frontend_mod};
use indexmap::IndexMap;
use pyo3::prelude::*;

use super::py_data_stores::PyDataStores;

#[pyclass]
pub struct PyFrontend {
    pub rc: Option<PyObject>,
    pub data_stores: Py<PyDataStores>,
    pub _users_: IndexMap<String, PyObject>,
    pub _spare_: IndexMap<String, PyObject>,
}

#[pymethods]
impl PyFrontend {
    #[getter]
    fn get_rc(&self) -> PyResult<Option<&PyObject>> {
        Ok(self.rc.as_ref())
    }

    #[getter]
    fn get_revision_control(&self) -> PyResult<Option<&PyObject>> {
        self.get_rc()
    }

    #[setter]
    fn set_rc(&mut self, rc: Option<&PyAny>) -> PyResult<()> {
        match rc {
            Some(obj) => Python::with_gil(|py| {
                self.rc = Some(obj.to_object(py));
            }),
            None => self.rc = None,
        }
        Ok(())
    }

    #[setter]
    fn set_revision_control(&mut self, rc: Option<&PyAny>) -> PyResult<()> {
        self.set_rc(rc)
    }

    #[getter]
    fn data_stores(&self) -> PyResult<&Py<PyDataStores>> {
        Ok(&self.data_stores)
    }
}

impl PyFrontend {
    pub fn new() -> Self {
        Self {
            rc: None,
            data_stores: Python::with_gil(|py| Py::new(py, PyDataStores::new())).unwrap(),
            _users_: IndexMap::new(),
            _spare_: IndexMap::new(),
        }
    }

    pub fn initialize() -> PyResult<()> {
        Python::with_gil(|py| {
            let fm = frontend_mod!(py);
            let f = Py::new(py, Self::new())?;
            fm.setattr(PY_FRONTEND, f)?;
            Ok(())
        })
    }
}

pub fn with_py_frontend<F, T>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, PyRef<PyFrontend>) -> PyResult<T>,
{
    if origen_metal::frontend::frontend_set()? {
        with_frontend_mod(|py, fm| {
            let py_fe = fm.getattr(PY_FRONTEND)?.extract::<PyRef<PyFrontend>>()?;
            func(py, py_fe)
        })
    } else {
        bail_with_runtime_error!("A frontend was requested but one has not been initialized!")
    }
}

pub fn with_mut_py_frontend<F, T>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, PyRefMut<PyFrontend>) -> PyResult<T>,
{
    if origen_metal::frontend::frontend_set()? {
        with_frontend_mod(|py, fm| {
            let py_fe = fm.getattr(PY_FRONTEND)?.extract::<PyRefMut<PyFrontend>>()?;
            func(py, py_fe)
        })
    } else {
        bail_with_runtime_error!("A frontend was requested but one has not been initialized!")
    }
}

pub fn with_required_rc<F, T>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, &PyObject) -> PyResult<T>,
{
    if origen_metal::frontend::frontend_set()? {
        with_frontend_mod(|py, fm| {
            let py_fe = fm.getattr(PY_FRONTEND)?.extract::<PyRef<PyFrontend>>()?;
            if let Some(rc) = py_fe.rc.as_ref() {
                func(py, rc)
            } else {
                bail_with_runtime_error!(
                    "A frontend revision control was requested but none has been set!"
                )
            }
        })
    } else {
        bail_with_runtime_error!("A frontend was requested but one has not been initialized!")
    }
}
