use crate::get_full_class_name;
use crate::meta::py_like_apis::dict_like_api::{DictLikeAPI, DictLikeIter};
use indexmap::map::IndexMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use super::pin_collection::PinCollection;
use origen::core::model::pins::Endianness;

#[pyclass]
pub struct PinContainer {
    pub model_id: usize,
}

#[pymethods]
impl PinContainer {
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

    #[args(names = "*", options = "**")]
    fn collect(&self, py: Python, names: &PyTuple, options: Option<&PyDict>) -> PyResult<Py<PinCollection>> {
        let mut endianness = Option::None;
        match options {
            Some(options) => {
                if let Some(opt) = options.get_item("little_endian") {
                    if opt.extract::<bool>()? {
                        endianness = Option::Some(Endianness::LittleEndian);
                    } else {
                        endianness = Option::Some(Endianness::BigEndian);
                    }
                }
            }
            None => {}
        }

        let mut name_strs: Vec<String> = vec![];
        for (_i, n) in names.iter().enumerate() {
            let cls = get_full_class_name(n)?;
            if cls == "re.Pattern" || cls == "_sre.SRE_Pattern" {
                let r = n.getattr("pattern").unwrap();
                name_strs.push(format!("/{}/", r));
            } else {
                let _n = n.extract::<String>()?;
                name_strs.push(_n.clone());
            }
        }
        let collection = PinCollection::new(name_strs, endianness)?;
        let c = Py::new(py, collection).unwrap();
        Ok(c)
    }

    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}

impl DictLikeAPI for PinContainer {
    fn lookup_key(&self) -> &str {
        &"pin_groups"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.get_model(self.model_id).unwrap().pin_groups.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> PyResult<PyObject> {
        Ok(Py::new(
            py,
            super::pin_group::PinGroup {
                model_id: model_id,
                name: name.to_string(),
            },
        )
        .unwrap()
        .to_object(py))
    }
}
