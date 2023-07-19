use origen_metal::utils::mailer::Maillists as OrigenMLS;
use origen_metal::utils::mailer::MaillistsTOMLConfig;
use super::maillist::Maillist as PyML;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyDict};
use std::path::PathBuf;

// TEST_NEEDED
pub const OM_MAILLISTS_CLASS_QP: &str = "origen_metal.utils.mailer.Maillists";

#[pyclass(subclass)]
pub struct Maillists {
    om_mls: OrigenMLS
}

#[pymethods]
impl Maillists {
    #[new]
    #[pyo3(signature=(n, *dirs, continue_on_error=false))]
    fn new(n: String, dirs: &PyTuple, continue_on_error: bool) -> PyResult<Self> {
        Ok(Self {
            om_mls: OrigenMLS::new(
                n, 
                if dirs.is_empty() {
                    None
                } else {
                    Some(dirs.iter().map( |d| d.extract::<PathBuf>()).collect::<PyResult<Vec<PathBuf>>>()?)
                },
                continue_on_error,
            )?
        })
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.om_mls.name.to_owned())
    }

    fn get(&self, key: &str) -> PyResult<Option<PyML>> {
        Ok(match self.om_mls.maillists.get(key) {
            Some(ml) => Some(PyML::from_om(ml.clone())),
            None => None,
        })
    }

    #[getter]
    fn maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let retn = PyDict::new(py);
        for (n, ml) in self.om_mls.maillists.iter() {
            retn.set_item(n.to_string(), Py::new(py, PyML::from_om(ml.clone()))?)?;
        }
        Ok(retn)
    }

    #[getter]
    fn directories(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut retn: Vec<PyObject> = vec!();
        for d in self.om_mls.directories.iter() {
            retn.push(crate::pypath!(py, d.display()));
        }
        Ok(retn)
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.om_mls.maillists.keys().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<PyML>> {
        Ok(self.om_mls.maillists.iter().map(|(_n, ml)| PyML::from_om(ml.clone())).collect())
    }

    fn items(&self) -> PyResult<Vec<(String, PyML)>> {
        Ok(self.om_mls.maillists.iter().map(|(n, ml)| (n.to_string(), PyML::from_om(ml.clone()))).collect())
    }

    fn maillists_for<'py>(&self, py: Python<'py>, audience: &str) -> PyResult<&'py PyDict> {
        let retn = PyDict::new(py);
        for (n, ml) in self.om_mls.maillists_for(audience)? {
            retn.set_item(n, Py::new(py, PyML::from_om((*ml).clone()))?)?;
        }
        Ok(retn)
    }

    #[getter]
    fn dev_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.maillists_for(py, "development")
    }

    #[getter]
    fn develop_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.dev_maillists(py)
    }

    #[getter]
    fn development_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.dev_maillists(py)
    }

    #[getter]
    fn prod_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.maillists_for(py, "production")
    }

    #[getter]
    fn production_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.prod_maillists(py)
    }

    #[getter]
    fn release_maillists<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.prod_maillists(py)
    }

    fn __getitem__(&self, key: &str) -> PyResult<PyML> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err({
                format!("No maillist {} is available!", key)
            }))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.om_mls.maillists.len())
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

impl Maillists {
    pub fn toml_config_into_args<'py>(py: Python<'py>, name: &str, continue_on_error: Option<bool>, config: &MaillistsTOMLConfig) -> PyResult<(&'py PyTuple, &'py PyDict)> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("continue_on_error", continue_on_error)?;
        let args = PyTuple::new(py, {
            let mut v = vec![];
            v.push(name.to_object(py));
            v.extend(config.resolve_dirs().iter().map(|d| d.to_object(py)).collect::<Vec<PyObject>>());
            v
        });
        Ok((args, kwargs))
    }
}