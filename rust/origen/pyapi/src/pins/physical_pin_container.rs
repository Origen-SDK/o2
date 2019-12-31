use origen::DUT;
use pyo3::prelude::*;
use pyo3::{exceptions};
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};
use pyo3::class::mapping::*;
use super::pin::Pin;

#[pyclass]
pub struct PhysicalPinContainer {
    pub path: String,
    pub model_id: usize,
}

#[pymethods]
impl PhysicalPinContainer {
    fn keys(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let ids = &model.physical_pins;

        let mut v: Vec<String> = Vec::new();
        for (n, _p) in ids {
            v.push(n.clone());
        }
        Ok(v)
    }

    fn values(&self) -> PyResult<Vec<Py<Pin>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let physical_pins = &model.physical_pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<Pin>> = Vec::new();
        for (n, _p) in physical_pins {
            v.push(Py::new(py, Pin {
                id: String::from(n.clone()),
                path: String::from(self.path.clone()),
                model_id: self.model_id,
            }).unwrap())
        }
        Ok(v)
    } 

    fn items(&self) -> PyResult<Vec<(String, Py<Pin>)>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        let pins = &model.physical_pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut items: Vec<(String, Py<Pin>)> = Vec::new();
        for (n, _p) in pins {
            items.push((
                n.clone(),
                Py::new(py, Pin {
                    id: String::from(n.clone()),
                    path: String::from(self.path.clone()),
                    model_id: self.model_id,
                }).unwrap(),
            ));
        }
        Ok(items)
    }

    #[getter]
    fn get_ids(&self) -> PyResult<Vec<String>> {
        self.keys()
    }
}

#[pyproto]
impl PyMappingProtocol for PhysicalPinContainer {
    fn __getitem__(&self, id: &str) -> PyResult<Py<Pin>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.physical_pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, Pin {
                  id: String::from(id),
                  path: String::from(&self.path),
                  model_id: self.model_id,
              }).unwrap())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin or pin alias found for {}", id)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model.number_of_physical_pins())
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PhysicalPinContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PhysicalPinContainerIter> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(slf.model_id)?;
        Ok(PhysicalPinContainerIter {
            keys: model.physical_pins.iter().map(|(s, _)| s.clone()).collect(),
            i: 0,
        })
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PhysicalPinContainer {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(self.model_id)?;
        Ok(model._contains(item))
    }
}

#[pyclass]
pub struct PhysicalPinContainerIter {
    keys: Vec<String>,
    i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PhysicalPinContainerIter {

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    /// The Iterator will be created with an index starting at 0 and the pin ids at the time of its creation.
    /// For each call to 'next', we'll create a pin object with the next value in the list, or None, if no more keys are available.
    /// Note: this means that the iterator can become stale if the PinContainer is changed. This can happen if the iterator is stored from Python code
    ///  directly. E.g.: i = dut.pins.__iter__() => iterator with the pin ids at the time of creation,
    /// Todo: Fix the above using iterators. My Rust skills aren't there yet though... - Coreyeng
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None)
        }
        let id = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(id))
    }
}
