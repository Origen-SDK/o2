use pyo3::class::mapping::*;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PyTuple};
use indexmap::map::IndexMap;
use crate::meta::py_like_apis::dict_like_api::{DictLikeAPI, DictLikeIter};

#[pyclass]
pub struct PhysicalPinContainer {
    pub model_id: usize,
}

#[pymethods]
impl PhysicalPinContainer {
    fn keys(&self) -> PyResult<Vec<String>> {
        DictLikeAPI::keys(self)
    }

    fn values(&self) -> PyResult<Vec<PyObject>> {
        DictLikeAPI::values(self)
    }

    fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
        DictLikeAPI::items(self)
    }

    fn get(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::get(self, name)
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        self.keys()
    }
}


impl DictLikeAPI for PhysicalPinContainer {
    fn lookup_key(&self) -> &str {
        &"pins"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.get_model(self.model_id).unwrap().pins.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> PyResult<PyObject> {
        Ok(Py::new(py, super::pin::Pin {model_id: model_id, name: name.to_string()}).unwrap().to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for PhysicalPinContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PhysicalPinContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}
