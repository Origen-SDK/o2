use origen::utility::mailer::Maillist as OrigenML;
use pyo3::prelude::*;
use std::collections::HashMap;

macro_rules! ml {
    ($name:expr) => {{
        crate::utility::mailer::maillist::Maillist {
            name: $name.to_string(),
        }
    }};
}

#[pyclass(subclass)]
pub struct Maillists {}

#[pymethods]
impl Maillists {
    fn get(&self, key: &str) -> PyResult<Option<Maillist>> {
        let ml = origen::maillists();
        Ok(match ml.maillists.get(key) {
            Some(_) => Some(ml!(key)),
            None => None,
        })
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let ml = origen::maillists();
        Ok(ml.maillists.keys().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<Maillist>> {
        let ml = origen::maillists();
        Ok(ml.maillists.iter().map(|(n, _)| ml!(n)).collect())
    }

    fn items(&self) -> PyResult<Vec<(String, Maillist)>> {
        let ml = origen::maillists();
        Ok(ml
            .maillists
            .iter()
            .map(|(n, _)| (n.to_string(), ml!(n)))
            .collect())
    }

    fn maillists_for(&self, audience: &str) -> PyResult<HashMap<String, Maillist>> {
        let m = origen::maillists();
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

    fn __getitem__(&self, key: &str) -> PyResult<Maillist> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err({
                format!("No maillist {} is available!", key)
            }))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let ml = origen::maillists();
        Ok(ml.maillists.len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<MaillistsIter> {
        Ok(MaillistsIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass]
pub struct MaillistsIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl MaillistsIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}

#[pyclass(subclass)]
pub struct Maillist {
    pub name: String,
}

impl Maillist {
    // Will return an error if the maillist doesn't exist
    pub fn with_origen_ml<T, F>(&self, func: F) -> origen::Result<T>
    where
        F: Fn(&OrigenML) -> origen::Result<T>,
    {
        let mlists = origen::maillists();
        let ml = mlists.get_maillist(&self.name)?;
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

    fn resolve_recipients(&self, domain: Option<String>) -> PyResult<Vec<String>> {
        Ok(self.with_origen_ml(|ml| {
            Ok(ml
                .resolve_recipients(&domain)?
                .iter()
                .map(|mb| mb.to_string())
                .collect::<Vec<String>>())
        })?)
    }

    // fn test(&self, message: Option<String>) -> PyResult<()> {
    //     // ...
    // }

    // fn send(&self, message: String) -> PyResult<()> {
    //     // ...
    // }
}
