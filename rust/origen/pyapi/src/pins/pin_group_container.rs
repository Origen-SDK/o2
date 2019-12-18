use origen::DUT;
use pyo3::prelude::*;
use pyo3::{exceptions};
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};
use pyo3::class::mapping::*;

use super::pin_group::PinGroup;

#[pyclass]
pub struct PinGroupContainer {
    pub path: String,
}

#[pyproto]
impl PyMappingProtocol for PinGroupContainer {
    fn __getitem__(&self, id: &str) -> PyResult<Py<PinGroup>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_group(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    id: String::from(id),
                    path: String::from(&self.path),
                }).unwrap())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", id)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.number_of_pin_groups())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroupContainer {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.contains_pin_group(item))
    }
}

#[pyclass]
struct PinGroupContainerIter {
    keys: Vec<String>,
    i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroupContainerIter {

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
