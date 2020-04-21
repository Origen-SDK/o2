use crate::meta::py_like_apis::dict_like_api::{DictLikeAPI, DictLikeIter};
use indexmap::map::IndexMap;
use origen::DUT;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct PinHeader {
    pub name: String,
    pub model_id: usize,
}

#[pymethods]
impl PinHeader {
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let h = dut._get_pin_header(self.model_id, &self.name)?;
        Ok(h.name.clone())
    }

    #[getter]
    fn pin_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let h = dut._get_pin_header(self.model_id, &self.name)?;
        Ok(h.pin_names.clone())
    }

    #[getter]
    fn physical_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let h = dut._get_pin_header(self.model_id, &self.name)?;
        Ok(dut.verify_names(self.model_id, &h.pin_names)?)
    }

    #[getter]
    fn width(&self) -> PyResult<usize> {
        Ok(self.physical_names()?.len())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___origen__model_id__(&self) -> PyResult<usize> {
        Ok(self.model_id)
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinHeader {
    // Need to overwrite contains to account for aliasing
    fn __len__(&self) -> PyResult<usize> {
        self.width()
    }
}

#[pyclass]
pub struct PinHeaderContainer {
    pub model_id: usize,
}

#[pymethods]
impl PinHeaderContainer {
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
}

impl DictLikeAPI for PinHeaderContainer {
    fn lookup_key(&self) -> &str {
        &"pin_headers"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.get_model(self.model_id).unwrap().pin_headers.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> PyResult<PyObject> {
        Ok(Py::new(
            py,
            PinHeader {
                model_id: model_id,
                name: name.to_string(),
            },
        )
        .unwrap()
        .to_object(py))
    }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for PinHeaderContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinHeaderContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}
