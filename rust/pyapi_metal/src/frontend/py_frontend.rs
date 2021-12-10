use super::{with_frontend_mod, PY_FRONTEND};
use crate::{bail_with_runtime_error, frontend_mod};
use pyo3::prelude::*;

#[pyclass]
pub struct PyFrontend {
    pub rc: Option<PyObject>,
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
}

impl PyFrontend {
    pub fn new() -> Self {
        Self { rc: None }
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

pub(crate) fn with_py_frontend<F, T>(mut func: F) -> PyResult<T>
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

pub(crate) fn with_required_rc<F, T>(mut func: F) -> PyResult<T>
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
