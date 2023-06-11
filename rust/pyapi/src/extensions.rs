use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use origen_metal::indexmap::IndexMap;
use crate::current_command::get_command;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "extensions")?;
    subm.add_class::<Extensions>()?;
    subm.add_class::<Extension>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass]
pub struct Extension {
    name: String,
    args: Py<PyDict>,
    source: String,
    ext_mod: Option<Py<PyModule>>,
}

#[pymethods]
impl Extension {
    #[getter]
    pub fn name(&self) -> PyResult<&str> {
        Ok(&self.name)
    }

    #[getter]
    pub fn args<'py>(&'py self, py: Python<'py>) -> PyResult<&'py PyDict> {
        Ok(self.args.as_ref(py))
    }

    #[getter]
    pub fn source(&self, py: Python) -> PyResult<PyObject> {
        Ok(pyapi_metal::pypath!(py, &self.source))
    }

    #[getter]
    pub fn r#mod(&self) -> PyResult<Option<&Py<PyModule>>> {
        Ok(self.ext_mod.as_ref())
    }

    #[getter]
    pub fn module(&self) -> PyResult<Option<&Py<PyModule>>> {
        Ok(self.ext_mod.as_ref())
    }
}

#[pyclass]
pub struct Extensions {
    exts: IndexMap<String, Py<Extension>>
}

#[pymethods]
impl Extensions {
    fn get(&self, ext_name: &str) -> PyResult<Option<&Py<Extension>>> {
        Ok(match self.exts.get(ext_name) {
            Some(ext) => Some(ext),
            None => None,
        })
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.exts.keys().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<&Py<Extension>>> {
        let mut retn: Vec<&Py<Extension>> = vec![];
        for (_, ext) in self.exts.iter() {
            retn.push(ext);
        }
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, &Py<Extension>)>> {
        let mut retn: Vec<(String, &Py<Extension>)> = vec![];
        for (n, ext) in self.exts.iter() {
            retn.push((n.to_string(), ext));
        }
        Ok(retn)
    }

    fn __getitem__(&self, py: Python, key: &str) -> PyResult<&Py<Extension>> {
        if let Some(s) = self.get(key)? {
            Ok(s)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(format!(
                "No extension '{}' available for command '{}'",
                key,
                get_command(py)?.cmd()?
            )))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.exts.len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ExtensionsIter> {
        Ok(ExtensionsIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass]
pub struct ExtensionsIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl ExtensionsIter {
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

impl Extensions {
    pub fn new<'py>(py: Python<'py>, exts: &PyList, ext_args: Py<PyDict>) -> PyResult<Self> {
        let mut slf = Self {
            exts: IndexMap::new(),
        };

        for ext in exts.iter() {
            let ext_cfg = ext.extract::<&PyDict>()?;
            let source = PyAny::get_item(ext_cfg, "source")?.extract::<String>()?;
            let ext_name;
            let ext_path;
            if source == "app" {
                ext_name = "app".to_string();
                ext_path = "app".to_string();
            } else {
                ext_name = PyAny::get_item(ext_cfg, "name")?.extract::<String>()?;
                ext_path = format!("{}.{}", source, ext_name);
            }
            let src_ext_args = PyAny::get_item(ext_args.as_ref(py), &source)?.extract::<&PyDict>()?;

            let py_ext = Extension {
                args: {
                    if source == "app" {
                        src_ext_args.into()
                    } else {
                        PyAny::get_item(src_ext_args, &ext_name)?.extract::<Py<PyDict>>()?
                    }
                },
                name: ext_name,
                source: ext_path.clone(),
                ext_mod: {
                    let m = PyAny::get_item(ext_cfg, "mod")?;
                    if m.is_none() {
                        None
                    } else {
                        Some(m.extract::<&PyModule>()?.into_py(py))
                    }
                }
            };
            slf.exts.insert(ext_path, Py::new(py, py_ext)?);
        }

        Ok(slf)
    }
}