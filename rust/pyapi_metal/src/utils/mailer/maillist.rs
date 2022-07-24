use pyo3::prelude::*;
use pyo3::types::{PyList, PyDict};
use std::path::PathBuf;
use origen_metal::utils::mailer::Maillist as OML;
use crate::prelude::{typed_value};

#[pyclass(subclass)]
pub struct Maillist {
    pub om_ml: OML
}

#[pymethods]
impl Maillist {
    #[staticmethod]
    pub fn from_file(f: PathBuf) -> PyResult<Self> {
        Ok(Self {
            om_ml: OML::from_file(&f)?
        })
    }

    #[new]
    pub fn new(name: String, recipients: &PyList, signature: Option<String>, audience: Option<String>, domain: Option<String>) -> PyResult<Self> {
        Ok(Self {
            om_ml: OML::new(
                name,
                recipients.iter().map(|r| r.extract::<String>()).collect::<PyResult<Vec<String>>>()?,
                signature,
                audience,
                domain
            )?
        })
    }

    #[getter]
    fn recipients(&self) -> PyResult<Vec<String>> {
        Ok(self.om_ml.recipients().to_vec())
    }

    #[getter]
    fn signature(&self) -> PyResult<Option<String>> {
        Ok(self.om_ml.signature().clone())
    }

    #[getter]
    fn audience(&self) -> PyResult<Option<String>> {
        Ok(self.om_ml.audience().clone())
    }

    #[getter]
    fn is_dev(&self) -> PyResult<bool> {
        Ok(self.om_ml.is_development())
    }

    #[getter]
    fn is_develop(&self) -> PyResult<bool> {
        self.is_dev()
    }

    #[getter]
    fn is_development(&self) -> PyResult<bool> {
        self.is_dev()
    }

    #[getter]
    fn is_prod(&self) -> PyResult<bool> {
        Ok(self.om_ml.is_production())
    }

    #[getter]
    fn is_production(&self) -> PyResult<bool> {
        self.is_prod()
    }

    #[getter]
    fn is_release(&self) -> PyResult<bool> {
        self.is_prod()
    }

    #[getter]
    fn domain(&self) -> PyResult<Option<String>> {
        Ok(self.om_ml.domain().clone())
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.om_ml.name.clone())
    }

    #[getter]
    fn file(&self, py: Python) -> PyResult<PyObject> {
        if let Some(f) = self.om_ml.file() {
            Ok(crate::pypath!(py, f.display()))
        } else {
            Ok(py.None())
        }
    }

    fn resolve_recipients(&self, domain: Option<String>) -> PyResult<Vec<String>> {
        Ok(self.om_ml.resolve_recipients(&domain)?.iter().map(|mb| mb.to_string()).collect::<Vec<String>>())
    }

    #[getter]
    fn config<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let c = typed_value::into_pydict(py, self.om_ml.config()?)?;
        c.set_item("file", self.file(py)?)?;
        Ok(c)
    }

    // TODO
    // fn test(&self, message: Option<String>) -> PyResult<()> {
    //     // ...
    // }

    // TODO
    // fn send(&self, message: String) -> PyResult<()> {
    //     // ...
    // }
}

impl Maillist {
    // NOTE: this is only allowed as long as Maillists stays immutable.
    // otherwise, multiple copies risk falling out of sync
    pub fn from_om(om_ml: OML) -> Self {
        Self {
            om_ml
        }
    }
}