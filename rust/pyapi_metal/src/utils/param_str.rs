use pyo3::prelude::*;
use pyo3::basic::CompareOp;
use pyo3::types::{PyDict, PyType};
use origen_metal::Result;
use origen_metal::utils::param_str::ParamStr as OmParamStr;
use origen_metal::utils::param_str::MultiParamStr as OmMultiParamStr;
use indexmap::IndexMap;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "param_str")?;
    subm.add_class::<ParamStr>()?;
    subm.add_class::<MultiParamStr>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass]
struct ParamStr {
    om: OmParamStr
}

#[pymethods]
impl ParamStr {
    // Create a new ParamStr, and parse it, in one method, returning the ParamStr as parsed
    #[classmethod]
    #[pyo3(signature=(input_str, allows_leading_str=false, defaults=None, allows_non_defaults=None))]
    fn and_parse<'py>(cls: &PyType, py: Python<'py>, input_str: String, allows_leading_str: bool, defaults: Option<&PyDict>, allows_non_defaults: Option<bool>) -> PyResult<Py<Self>> {
        let slf = Self {om: Self::new_om(allows_leading_str, defaults, allows_non_defaults)?};
        let obj = Py::new(cls.py(), slf)?;
        {
            let pyref = obj.borrow_mut(py);
            Self::parse(pyref, input_str)?;
        }
        Ok(obj)
    }

    #[new]
    #[pyo3(signature=(allows_leading_str=false, defaults=None, allows_non_defaults=None))]
    fn new(allows_leading_str: bool, defaults: Option<&PyDict>, allows_non_defaults: Option<bool>) -> PyResult<Self> {
        Ok(Self {om: Self::new_om(allows_leading_str, defaults, allows_non_defaults)?})
    }

    #[getter]
    pub fn parsed<'py>(&'py self, py: Python<'py>) -> PyResult<Option<&PyDict>> {
        if let Some(parsed) = self.om.parsed() {
            let pyd = PyDict::new(py);
            for (k, v) in parsed {
                pyd.set_item(k, v)?;
            }
            Ok(Some(pyd))
        } else {
            Ok(None)
        }
    }

    #[getter]
    pub fn allows_non_defaults(&self) -> PyResult<Option<bool>> {
        if let Some(exp) = self.om.allows_non_defaults() {
            Ok(Some(exp))
        } else {
            Ok(None)
        }
    }

    #[getter]
    pub fn defaults<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyDict>> {
        if let Some(defs) = self.om.defaults() {
            let pyd = PyDict::new(py);
            for (k, v) in defs {
                pyd.set_item(k, v)?;
            }
            Ok(Some(pyd))
        } else {
            Ok(None)
        }
    }

    pub fn parse(mut slf: PyRefMut<Self>, input: String) -> PyResult<PyRefMut<Self>> {
        slf.om.parse(input)?;
        Ok(slf)
    }

    #[getter]
    pub fn raw(&self) -> PyResult<Option<String>> {
        Ok(self.om.raw()?.to_owned())
    }

    #[getter]
    pub fn leading(&self) -> PyResult<Option<String>> {
        Ok(self.om.leading()?.to_owned())
    }

    #[getter]
    pub fn allows_leading_str(&self) -> PyResult<bool> {
        Ok(self.om.allows_leading_str())
    }

    fn to_str(&self) -> PyResult<String> {
        Ok(self.om.to_string()?)
    }

    fn __str__(&self) -> PyResult<String> {
        self.to_str()
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.om.keys()?.iter().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<Vec<String>>> {
        Ok(self.om.values()?.iter().map(|v| (*v).clone()).collect())
    }

    fn items(&self) -> PyResult<Vec<(String, Vec<String>)>> {
        Ok(self.om.get_parsed()?.iter().map(|(k, v)| (k.to_string(), (*v).clone())).collect())
    }

    fn get(&self, key: &str) -> PyResult<Option<Vec<String>>> {
        if let Some(val) = self.om.get(key)? {
            Ok(Some((*val).clone()))
        } else {
            Ok(None)
        }
    }

    fn __getitem__(&self, key: &str) -> PyResult<Vec<String>> {
        if let Some(s) = self.get(key)? {
            Ok(s)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(format!("No key '{}'", key)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.om.get_parsed()?.len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ParamStrIter> {
        Ok(ParamStrIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let py_other = match other.extract::<PyRef<Self>>() {
            Ok(o) => o,
            Err(_) => return Python::with_gil(|py| Ok(false.to_object(py))),
        };

        Python::with_gil(|py| match op {
            CompareOp::Eq => Ok((py_other.om == self.om).to_object(py)),
            CompareOp::Ne => Ok((py_other.om != self.om).to_object(py)),
            _ => Ok(py.NotImplemented()),
        })
    }
}

impl ParamStr {
    fn from_om(om_param_str: OmParamStr) -> Self {
        Self {
            om: om_param_str
        }
    }

    fn new_om(allows_leading_str: bool, defaults: Option<&PyDict>, allows_non_defaults: Option<bool>) -> Result<OmParamStr> {
        let om_defaults;
        if let Some(defs) = defaults {
            let mut om_defs = IndexMap::new();
            for (key, default) in defs {
                let def;
                if let Ok(s) = default.extract::<String>() {
                    def = Some(vec!(s));
                } else if let Ok(v) = default.extract::<Vec<String>>() {
                    def = Some(v);
                } else if default.is_none() {
                    def = None
                } else {
                    bail!("ParamStr default value must be either None, a str, or list of strs");
                }
                om_defs.insert(key.extract::<String>()?, def);
            }
            om_defaults = Some((allows_non_defaults.unwrap_or(false), om_defs))
        } else {
            om_defaults = None
        }
        Ok(OmParamStr::new(allows_leading_str, om_defaults))
    }
}

#[pyclass]
pub struct ParamStrIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl ParamStrIter {
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

#[pyclass]
struct MultiParamStr {
    om: OmMultiParamStr
}

#[pymethods]
impl MultiParamStr {

    #[new]
    #[pyo3(signature=(allow_leading_str=false))]
    fn new(allow_leading_str: bool) -> PyResult<Self> {
        let om = OmMultiParamStr::new(allow_leading_str);
        Ok(Self {om: om})
    }

    #[getter]
    pub fn raw(&self) -> PyResult<Option<String>> {
        Ok(self.om.raw().to_owned())
    }

    #[getter]
    pub fn allows_leading_str(&self) -> PyResult<bool> {
        Ok(self.om.allows_leading_str())
    }

    pub fn parse(&mut self, multi_param_str: String) -> PyResult<Option<Vec<ParamStr>>> {
        self.om.parse(multi_param_str)?;
        self.parsed()
    }

    #[getter]
    pub fn parsed(&self) -> PyResult<Option<Vec<ParamStr>>> {
        Ok(self.om.parsed().as_ref().map(|param_strs| { param_strs.iter().map(|param_str| {
            ParamStr::from_om(param_str.clone())
        }).collect()}))
    }

    #[getter]
    pub fn leading(&self) -> PyResult<Option<String>> {
        Ok(self.om.leading().to_owned())
    }

    #[getter]
    pub fn param_strs(&self) -> PyResult<Vec<ParamStr>> {
        Ok(self.om.param_strs().iter().map(|param_str| {
            ParamStr::from_om(param_str.clone())
        }).collect())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.om.parsed().as_ref().map_or(0, |param_str| param_str.len()))
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<MultiParamStrIter> {
        Ok(MultiParamStrIter {
            len: slf.__len__()?,
            i: 0,
        })
    }
}

#[pyclass]
pub struct MultiParamStrIter {
    pub len: usize,
    pub i: usize,
}

#[pymethods]
impl MultiParamStrIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<usize>> {
        if slf.i >= slf.len {
            return Ok(None);
        }
        slf.i += 1;
        Ok(Some(slf.i))
    }
}