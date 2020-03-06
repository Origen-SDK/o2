use super::super::meta::py_like_apis::dict_like_api::{DictLikeAPI, DictLikeIter};
use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use super::super::timesets::*;
use indexmap::map::IndexMap;
use origen::error::Error;
use pyo3::class::mapping::*;
//use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

#[macro_export]
macro_rules! pytimeset_container {
    ($py:expr, $model_id:expr) => {
        Py::new(
            $py,
            TimesetContainer {
                model_id: $model_id,
            },
        )
        .unwrap()
    };
}

#[pyclass]
pub struct TimesetContainer {
    pub model_id: usize,
}

#[pymethods]
impl TimesetContainer {
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

impl DictLikeAPI for TimesetContainer {
    fn lookup_key(&self) -> &str {
        &"timesets"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.get_model(self.model_id).unwrap().timesets.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
        Ok(Py::new(py, super::timeset::Timeset::new(name, model_id))
            .unwrap()
            .to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for TimesetContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for TimesetContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}

#[macro_export]
macro_rules! pywavetable_container {
    ($py:expr, $model_id:expr, $timeset_id:expr, $timeset_name:expr) => {
        Py::new(
            $py,
            WavetableContainer {
                model_id: $model_id,
                timeset_id: $timeset_id,
                timeset_name: String::from($timeset_name),
            },
        )
        .unwrap()
    };
}

/// WaveTable Container
#[pyclass]
pub struct WavetableContainer {
    pub model_id: usize,
    pub timeset_id: usize,
    pub timeset_name: String,
}

#[pymethods]
impl WavetableContainer {
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

impl DictLikeAPI for WavetableContainer {
    fn lookup_key(&self) -> &str {
        &"wavetables"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.timesets[self.timeset_id].wavetable_ids.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
        Ok(Py::new(
            py,
            super::timeset::Wavetable::new(model_id, self.timeset_id, name),
        )
        .unwrap()
        .to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for WavetableContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for WavetableContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}

#[macro_export]
macro_rules! pywave_group_container {
    ($py:expr, $model_id:expr, $timeset_id:expr, $wavetable_id:expr, $name:expr) => {
        Py::new(
            $py,
            WaveGroupContainer {
                model_id: $model_id,
                timeset_id: $timeset_id,
                wavetable_id: $wavetable_id,
                name: String::from($name),
            },
        )
        .unwrap()
    };
}

/// Wave Group Container
#[pyclass]
pub struct WaveGroupContainer {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub name: String,
}

#[pymethods]
impl WaveGroupContainer {
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

impl DictLikeAPI for WaveGroupContainer {
    fn lookup_key(&self) -> &str {
        &"wave_groups"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.wavetables[self.wavetable_id].wave_group_ids.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
        Ok(Py::new(
            py,
            super::timeset::WaveGroup::new(model_id, self.timeset_id, self.wavetable_id, name),
        )
        .unwrap()
        .to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for WaveGroupContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for WaveGroupContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}

#[macro_export]
macro_rules! pywave_container {
    ($py:expr, $model_id:expr, $timeset_id:expr, $wavetable_id:expr, $wave_group_id:expr, $name:expr) => {
        Py::new(
            $py,
            WaveContainer {
                model_id: $model_id,
                timeset_id: $timeset_id,
                wavetable_id: $wavetable_id,
                wave_group_id: $wave_group_id,
                name: String::from($name),
            },
        )
        .unwrap()
    };
}

/// Wave Container
#[pyclass]
pub struct WaveContainer {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub wave_group_id: usize,
    pub name: String,
}

#[pymethods]
impl WaveContainer {
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

    fn applied_to(&self, pin: String) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let wgrp = &dut.wave_groups[self.wave_group_id];

        let gil = Python::acquire_gil();
        let py = gil.python();
        let rtn = PyDict::new(py);
        for name in wgrp.get_waves_applied_to(&dut, &pin).iter() {
            rtn.set_item(name.clone(), self.new_pyitem(py, &name, self.model_id)?)?;
        }
        Ok(rtn.to_object(py))
    }
}

impl DictLikeAPI for WaveContainer {
    fn lookup_key(&self) -> &str {
        &"waves"
    }

    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize> {
        dut.wave_groups[self.wave_group_id].wave_ids.clone()
    }

    fn model_id(&self) -> usize {
        self.model_id
    }

    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error> {
        Ok(Py::new(
            py,
            super::timeset::Wave::new(
                model_id,
                self.timeset_id,
                self.wavetable_id,
                self.wave_group_id,
                name,
            ),
        )
        .unwrap()
        .to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for WaveContainer {
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        DictLikeAPI::__getitem__(self, name)
    }

    fn __len__(&self) -> PyResult<usize> {
        DictLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for WaveContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
        DictLikeAPI::__iter__(&*slf)
    }
}

#[macro_export]
macro_rules! pyevent_container {
    ($py:expr, $model_id:expr, $timeset_id:expr, $wavetable_id:expr, $wave_group_id:expr, $wave_id:expr, $wave_name:expr) => {
        Py::new(
            $py,
            EventContainer {
                model_id: $model_id,
                timeset_id: $timeset_id,
                wavetable_id: $wavetable_id,
                wave_group_id: $wave_group_id,
                wave_id: $wave_id,
                wave_name: String::from($wave_name),
            },
        )
        .unwrap()
    };
}

/// EventList Container
#[pyclass]
#[derive(Clone)]
pub struct EventContainer {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub wave_group_id: usize,
    pub wave_id: usize,
    pub wave_name: String,
}

#[pymethods]
impl EventContainer {}

impl ListLikeAPI for EventContainer {
    fn item_ids(&self, dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
        dut.waves[self.wave_id].events.clone()
    }

    fn new_pyitem(&self, py: Python, idx: usize) -> Result<PyObject, Error> {
        Ok(Py::new(
            py,
            super::timeset::Event::new(
                self.model_id,
                self.timeset_id,
                self.wavetable_id,
                self.wave_group_id,
                self.wave_id,
                &self.wave_name,
                idx,
            ),
        )
        .unwrap()
        .to_object(py))
    }

    fn __iter__(&self) -> PyResult<ListLikeIter> {
        Ok(ListLikeIter {
            parent: Box::new((*self).clone()),
            i: 0,
        })
    }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for EventContainer {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        ListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        ListLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for EventContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
        ListLikeAPI::__iter__(&*slf)
    }
}
