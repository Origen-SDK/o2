use pyo3::prelude::*;
use indexmap::IndexMap;
use crate::_helpers::{indexmap_to_pydict, new_py_obj, pytype_from_pyany};
use crate::{runtime_error, key_error, type_error};
use pyo3::types::{PyDict, PyTuple, PyList, PyString, PyBool};
use pyo3::class::mapping::PyMappingProtocol;
use origen_metal::Result as OMResult;
use super::_frontend::DataStoreCategoryFrontend;

#[pyclass]
pub struct PyDataStores {
    categories: IndexMap<String, Py<PyDataStoreCategory>>
}

#[pymethods]
impl PyDataStores {
    #[getter]
    fn categories(&self, py: Python) -> PyResult<Py<PyDict>> {
        indexmap_to_pydict(py, &self.categories)
    }

    pub fn add_category(&mut self, py: Python, category: &str) -> PyResult<&Py<PyDataStoreCategory>> {
        if self.categories.contains_key(category) {
            runtime_error!(format!(
                "Data store category '{}' is already present",
                category
            ))
        } else {
            self.categories.insert(category.to_string(), PyDataStoreCategory::new_py(py, category)?);
            Ok(self.categories.get(category).unwrap())
        }
    }

    pub fn remove_category(&mut self, py: Python, category: &str) -> PyResult<()> {
        match self.categories.remove(category) {
            Some(py_cat) => {
                let cat = &mut *py_cat.borrow_mut(py);
                cat.mark_stale(py)?;
                Ok(())
            },
            None => runtime_error!(format!(
                "Cannot remove non-existent data store category '{}'",
                category
            ))
        }
    }

    pub fn get(&self, key: &str) -> PyResult<Option<&Py<PyDataStoreCategory>>> {
        Ok(if let Some(cat) = self.categories.get(key) {
            Some(cat)
        } else {
            None
        })
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.categories.keys().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<&Py<PyDataStoreCategory>>> {
        Ok(self.categories.iter().map(|(_, cat)| cat).collect())
    }

    fn items(&self) -> PyResult<Vec<(String, &Py<PyDataStoreCategory>)>> {
        Ok(self.categories.iter().map(|(n, cat)| (n.to_string(), cat)).collect())
    }
}

#[pyproto]
impl PyMappingProtocol for PyDataStores {
    fn __getitem__(&self, key: &str) -> PyResult<&Py<PyDataStoreCategory>> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            key_error!(format!("Unknown data store category '{}'", key))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.categories.len())
    }
}

#[pyclass]
pub struct PyDataStoresIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PyDataStoresIter {
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

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PyDataStores {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyDataStoresIter> {
        Ok(PyDataStoresIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

impl PyDataStores {
    pub fn new() -> Self {
        Self {
            categories: IndexMap::new()
        }
    }

    pub fn rusty_categories(&self) -> &IndexMap<String, Py<PyDataStoreCategory>> {
        &self.categories
    }
}

#[pyclass]
pub struct PyDataStoreCategory {
    name: String,
    objects: IndexMap<String, PyObject>,
    stale: bool
}

#[pymethods]
impl PyDataStoreCategory {

    #[getter]
    fn name(&self) -> PyResult<&str> {
        self.check_stale()?;
        Ok(&self.name)
    }

    // Note: This will shallow-copy ``init_args`` and ``init_kwargs``, if given.
    #[args(func_kwargs="**")]
    pub fn add(&mut self, py: Python, name: &str, cls: &PyAny, init_args: Option<&PyList>, init_kwargs: Option<&PyDict>, func_kwargs: Option<&PyDict>) -> PyResult<&PyObject> {
        self.check_stale()?;
        if self.objects.contains_key(name) {
            runtime_error!(format!(
                "Data store '{}' is already present in category '{}'",
                name,
                self.name
            ))
        } else {
            let mut provide_name = false;
            let mut provide_category = false;
            let mut name_idx: Option<usize> = None;
            let mut category_idx: Option<usize> = None;
            if let Some(fa) = func_kwargs {
                if let Some(pn) = fa.get_item("provide_name") {
                    if pn.is_instance::<PyBool>()? {
                        provide_name = pn.extract::<bool>()?;
                    } else {
                        if let Ok(i) = pn.extract::<usize>() {
                            provide_name = true;
                            name_idx = Some(i);
                        } else {
                            return type_error!("Cannot interpret 'provide_name' as a bool or integer");
                        }
                    }
                }
                if let Some(pc) = fa.get_item("provide_category") {
                    if pc.is_instance::<PyBool>()? {
                        provide_category = pc.extract::<bool>()?;
                    } else {
                        if let Ok(i) = pc.extract::<usize>() {
                            provide_category= true;
                            category_idx = Some(i);
                        } else {
                            return type_error!("Cannot interpret 'provide_category' as a bool or integer");
                        }
                    }
                }
            }
            
            let mut add_args: Vec<&PyAny> = vec!();
            let add_kwargs;
            if let Some(kw) = init_kwargs {
                add_kwargs = kw.copy()?;
            } else {
                add_kwargs = PyDict::new(py);
            }
            if let Some(a) = init_args {
                add_args.append(&mut a.iter().collect::<Vec<&PyAny>>());
            }
            if provide_name {
                let add_name = PyString::new(py, name).as_ref();
                if let Some(i) = name_idx {
                    if i > add_args.len() {
                        return runtime_error!(format!("'provide_name' insert index {} exceeds argument list size {}", i, add_args.len()));
                    }
                    add_args.insert(i, add_name);
                } else {
                    if init_kwargs.is_some() && add_kwargs.contains("name")? {
                        return runtime_error!("'name' key is already present in keyword arguments");
                    }
                    add_kwargs.set_item("name", add_name)?;
                }
            }
            if provide_category {
                let add_cat = PyString::new(py, &self.name).as_ref();
                if let Some(i) = category_idx {
                    if i > add_args.len() {
                        return runtime_error!(format!("'provide_category' insert index {} exceeds argument list size {}", i, add_args.len()));
                    }
                    add_args.insert(i, add_cat);
                } else {
                    if init_kwargs.is_some() && add_kwargs.contains("category")? {
                        return runtime_error!("'category' key is already present in keyword arguments");
                    }
                    add_kwargs.set_item("category", add_cat)?;
                }
            }

            let new_ds = new_py_obj(py, pytype_from_pyany(py, cls)?, Some(PyTuple::new(py, add_args)), Some(add_kwargs))?.to_object(py);
            new_ds.call_method1(py, "_set_name_", (name,))?;
            new_ds.call_method1(py, "_set_category_", (&self.name,))?;
            self.objects.insert(
                name.to_string(),
                new_ds
            );
            Ok(self.objects.get(name).unwrap())
        }
    }

    pub fn get(&self, key: &str) -> PyResult<Option<&PyObject>> {
        Ok(self.objects()?.get(key))
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.objects()?.keys().map(|k| k.to_string()).collect::<Vec<String>>())
    }

    /// Note: slightly changing vs. the rust trait to match how Python's dictionaries behave.
    /// The Rust trait prototype is based on how Rust's HashMap behaves.
    pub fn remove(&mut self, py: Python, name: &str) -> PyResult<()> {
        self.check_stale()?;
        match self.objects.remove(name) {
            Some(py_ds) => {
                py_ds.call_method0(py, "_mark_stale_")?;
                Ok(())
            },
            None => runtime_error!(format!(
                "Cannot remove data store '{} as category '{}' does not contain this data store",
                name,
                &self.name
            ))
        }
    }

    fn values(&self, py: Python) -> PyResult<Vec<PyObject>> {
        Ok(self.objects()?.iter().map(|(_, obj)| obj.to_object(py)).collect::<Vec<PyObject>>())
    }

    fn items(&self, py: Python) -> PyResult<Vec<(String, PyObject)>> {
        Ok(self.objects()?.iter().map(|(n, obj)| (n.to_string(), obj.to_object(py))).collect::<Vec<(String, PyObject)>>())
    }
}

impl PyDataStoreCategory {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            objects: IndexMap::new(),
            stale: false
        }
    }

    pub fn new_py(py: Python, name: &str) -> PyResult<Py<Self>> {
        Py::new(py, Self::new(name))
    }

    pub fn into_frontend(&self) -> OMResult<DataStoreCategoryFrontend> {
        self.check_stale()?;
        Ok(DataStoreCategoryFrontend::new(&self.name))
    }

    pub fn check_stale(&self) -> PyResult<()> {
        if self.stale {
            runtime_error!(format!(
                "Stale category '{}' encountered",
                self.name
            ))
        } else {
            Ok(())
        }
    }

    pub fn objects(&self) -> PyResult<&IndexMap<String, PyObject>> {
        self.check_stale()?;
        Ok(&self.objects)
    }

    pub fn mark_stale(&mut self, py: Python) -> PyResult<()> {
        self.check_stale()?;
        for (_, py_ds) in &self.objects {
            py_ds.call_method0(py, "_mark_orphaned_")?;
        }
        self.stale = true;
        Ok(())
    }
}

#[pyproto]
impl PyMappingProtocol for PyDataStoreCategory {
    fn __getitem__(&self, key: &str) -> PyResult<&PyObject> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            key_error!(format!("'{}' is not in data store '{}'", key, &self.name))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.objects()?.len())
    }
}

#[pyclass]
pub struct PyDataStoreCategoryIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PyDataStoreCategoryIter {
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

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PyDataStoreCategory {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyDataStoreCategoryIter> {
        Ok(PyDataStoreCategoryIter {
            keys: slf.keys()?,
            i: 0,
        })
    }
}
