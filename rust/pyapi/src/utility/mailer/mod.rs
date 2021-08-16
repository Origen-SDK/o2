pub mod _frontend;

use origen::utility::mailer::Maillist as OrigenML;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;

#[pymodule]
fn mailer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Mailer>()?;
    m.add_class::<Maillist>()?;
    m.add_wrapped(wrap_pyfunction!(_mailer))?;
    Ok(())
}

#[pyfunction]
pub fn _mailer() -> PyResult<Mailer> {
    Ok(Mailer {})
}

/// Simple Python class that wraps the Origen's mailer
#[pyclass(subclass)]
pub struct Mailer {}

#[pymethods]
impl Mailer {
    #[getter]
    fn get_server(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.get_server()?)
    }

    #[getter]
    fn get_port(&self) -> PyResult<Option<usize>> {
        let m = origen::mailer();
        Ok(m.port)
    }

    #[getter]
    fn get_domain(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        Ok(m.domain.clone())
    }

    #[getter]
    fn get_auth_method(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.auth_method.to_string())
    }

    #[getter]
    fn get_timeout_seconds(&self) -> PyResult<u64> {
        let m = origen::mailer();
        Ok(m.timeout_seconds)
    }

    #[getter]
    fn get_timeout(&self) -> PyResult<u64> {
        let m = origen::mailer();
        Ok(m.timeout_seconds)
    }

    #[getter]
    fn get_service_user(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        if let Some(su) = m.service_user()? {
            Ok(Some(su.0.to_string()))
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn get_username(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.username()?)
    }

    #[getter]
    fn get_password(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.password()?)
    }

    #[getter]
    fn get_sender(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.sender()?)
    }

    #[getter]
    fn get_dataset(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        Ok(m.dataset()?)
    }

    // fn test(&self, to: Option<Vec<&PyAny>>) -> PyResult<()> {
    fn test(&self) -> PyResult<()> {
        let m = origen::mailer();
        Ok(m.test(None)?)
    }

    // fn send(&self, message: String, to: Option<Vec<String>>, audience: Option<String>) -> PyResult<()> {
    //     //
    // }

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

    // --- Maillist Related Methods ---

    #[getter]
    fn maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        let m = origen::mailer();
        let mut retn = HashMap::new();
        for name in m.maillists.keys() {
            retn.insert(
                name.to_string(),
                Maillist {
                    name: name.to_string(),
                },
            );
        }
        Ok(retn)
    }

    fn maillists_for(&self, audience: &str) -> PyResult<HashMap<String, Maillist>> {
        let m = origen::mailer();
        let mut retn = HashMap::new();
        for name in m.maillists_for(audience)?.keys() {
            retn.insert(
                name.to_string(),
                Maillist {
                    name: name.to_string(),
                },
            );
        }
        Ok(retn)
    }

    // --- Enumerated audience maillists ---

    #[getter]
    fn dev_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("development")
    }

    #[getter]
    fn develop_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("development")
    }

    #[getter]
    fn development_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("development")
    }

    #[getter]
    fn release_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("production")
    }

    #[getter]
    fn prod_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("production")
    }

    #[getter]
    fn production_maillists(&self) -> PyResult<HashMap<String, Maillist>> {
        self.maillists_for("production")
    }
}

#[pyclass(subclass)]
pub struct Maillist {
    name: String,
}

impl Maillist {
    // Will return an error if the maillist doesn't exist
    pub fn with_origen_ml<T, F>(&self, func: F) -> origen::Result<T>
    where
        F: Fn(&OrigenML) -> origen::Result<T>,
    {
        let m = origen::mailer();
        let ml = m.get_maillist(&self.name)?;
        func(ml)
    }
}

#[pymethods]
impl Maillist {
    #[getter]
    fn recipients(&self) -> PyResult<Vec<String>> {
        Ok(self.with_origen_ml(|ml| Ok(ml.recipients().to_vec()))?)
    }

    #[getter]
    fn signature(&self) -> PyResult<Option<String>> {
        Ok(self.with_origen_ml(|ml| Ok(ml.signature().clone()))?)
    }

    #[getter]
    fn audience(&self) -> PyResult<Option<String>> {
        Ok(self.with_origen_ml(|ml| Ok(ml.audience().clone()))?)
    }

    #[getter]
    fn domain(&self) -> PyResult<Option<String>> {
        Ok(self.with_origen_ml(|ml| Ok(ml.domain().clone()))?)
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.with_origen_ml(|ml| Ok(ml.name.clone()))?)
    }

    #[getter]
    fn file(&self) -> PyResult<PyObject> {
        Ok(self.with_origen_ml(|ml| {
            let gil = Python::acquire_gil();
            let py = gil.python();
            if let Some(f) = ml.file() {
                Ok(crate::pypath!(py, f.display()))
            } else {
                Ok(py.None())
            }
        })?)
    }

    fn resolve_recipients(&self) -> PyResult<Vec<String>> {
        let m = origen::mailer();
        let mailer_domain = &m.domain;
        let ml = m.get_maillist(&self.name)?;
        Ok(ml
            .resolve_recipients(mailer_domain)?
            .iter()
            .map(|mailbox| mailbox.to_string())
            .collect::<Vec<String>>())
    }

    // fn test(&self, message: Option<String>) -> PyResult<()> {
    //     // ...
    // }

    // fn send(&self, message: String) -> PyResult<()> {
    //     // ...
    // }
}
