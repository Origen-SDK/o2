pub mod _frontend;
pub mod maillist;

use super::app_utility;
use maillist::{Maillist, Maillists};
use origen::utility::mailer::Mailer as OMailer;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use pyapi_metal::prelude::{PyOutcome, typed_value};

#[pymodule]
pub fn mailer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Mailer>()?;
    m.add_class::<Maillist>()?;
    m.add_class::<Maillists>()?;
    m.add_wrapped(wrap_pyfunction!(_mailer))?;
    m.add_wrapped(wrap_pyfunction!(maillists))?;
    Ok(())
}

#[pyfunction]
pub fn _mailer() -> PyResult<Option<PyObject>> {
    let c = &origen::ORIGEN_CONFIG;
    let m = app_utility(
        "mailer",
        c.mailer.as_ref(),
        Some("origen.utility.mailer.Mailer"),
        false,
    );
    match m {
        Ok(_) => m,
        Err(e) => {
            log_error!("Error creating mailer. No mailer will be available.",);
            let gil = Python::acquire_gil();
            let py = gil.python();
            e.print(py);
            Ok(None)
        }
    }
}

#[pyfunction]
pub fn maillists() -> PyResult<Maillists> {
    Ok(Maillists {})
}

/// Simple Python class that wraps the Origen's mailer
#[pyclass(subclass)]
pub struct Mailer {
    mailer: OMailer,
}

#[pymethods]
impl Mailer {
    #[new]
    #[args(config = "**")]
    fn new(config: Option<&PyDict>) -> PyResult<Self> {
        let mut c: HashMap<String, String> = HashMap::new();
        if let Some(cfg) = config {
            for (k, v) in cfg {
                c.insert(k.extract::<String>()?, v.extract::<String>()?);
            }
        }
        Ok(Self {
            mailer: OMailer::new(&c)?,
        })
    }

    #[getter]
    fn config<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        typed_value::into_pydict(py, self.mailer.config()?)
    }

    #[getter]
    fn get_server(&self) -> PyResult<String> {
        Ok(self.mailer.server.clone())
    }

    #[getter]
    fn get_port(&self) -> PyResult<Option<usize>> {
        Ok(self.mailer.port)
    }

    #[getter]
    fn get_domain(&self) -> PyResult<Option<String>> {
        Ok(self.mailer.domain.clone())
    }

    #[getter]
    fn get_auth_method(&self) -> PyResult<String> {
        Ok(self.mailer.auth_method.to_string())
    }

    #[getter]
    fn get_timeout_seconds(&self) -> PyResult<u64> {
        Ok(self.mailer.timeout_seconds)
    }

    #[getter]
    fn get_timeout(&self) -> PyResult<u64> {
        Ok(self.mailer.timeout_seconds)
    }

    #[getter]
    fn get_service_user(&self) -> PyResult<Option<String>> {
        if let Some(su) = self.mailer.service_user()? {
            Ok(Some(su.0.to_string()))
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn get_username(&self) -> PyResult<String> {
        Ok(self.mailer.username()?)
    }

    #[getter]
    fn get_password(&self) -> PyResult<String> {
        Ok(self.mailer.password()?)
    }

    #[getter]
    fn get_sender(&self) -> PyResult<String> {
        Ok(self.mailer.sender()?)
    }

    #[getter]
    fn get_dataset(&self) -> PyResult<Option<String>> {
        Ok(self.mailer.dataset()?)
    }

    fn test(&self, to: Option<Vec<&str>>) -> PyResult<PyOutcome> {
        Ok(PyOutcome::from_origen(self.mailer.test(to)?))
    }

    fn send(
        &self,
        to: Vec<&str>,
        body: Option<&str>,
        subject: Option<&str>,
    ) -> PyResult<PyOutcome> {
        let e = origen_metal::require_current_user_email()?;
        let m = self.mailer.compose(&e, to, subject, body, true)?;
        Ok(PyOutcome::from_origen(self.mailer.send(m)?))
    }

    // #[getter]
    // fn signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn origen_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn app_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn release_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }
}
