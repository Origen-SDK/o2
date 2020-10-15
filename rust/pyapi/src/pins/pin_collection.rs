use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use origen::core::model::pins::pin_store::PinStore as OrigenPinCollection;
use origen::core::model::pins::Endianness;
use origen::error::Error;
use origen::{dut, DUT};
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};
use super::pin_actions::PinActions;
use num_bigint::BigUint;
use origen::Transaction;
use super::super::pins::extract_pin_transaction;

#[pyclass]
#[derive(Clone)]
pub struct PinCollection {
    pin_collection: OrigenPinCollection,
}

impl PinCollection {
    pub fn new(
        // Todo - allow this to take Pins as well
        names: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<PinCollection, Error> {
        let dut = dut();
        let collection = dut.collect(&names.iter().map(|n| (n.to_string(), 0)).collect(), endianness)?;
        Ok(PinCollection {
            pin_collection: collection,
        })
    }

    pub fn from_ids_unchecked(pin_ids: Vec<usize>, endianness: Option<Endianness>) -> Self {
        PinCollection {
            pin_collection: OrigenPinCollection {
                pin_ids: pin_ids,
                endianness: endianness.unwrap_or(Endianness::LittleEndian)
            }
        }
    }
}

#[pymethods]
impl PinCollection {
    #[setter]
    fn actions(slf: PyRefMut<Self>, actions: &PyAny) -> PyResult<()> {
        Self::set_actions(slf, actions, None)?;
        Ok(())
    }

    #[args(kwargs = "**")]
    fn set_actions(slf: PyRefMut<Self>, actions: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        slf.pin_collection.update(
            &dut,
            &extract_pin_transaction(actions, kwargs)?
        )?;
        Ok(slf.into())
    }

    #[getter]
    fn get_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let pin_actions = self.pin_collection.get_actions(&dut)?;
        Ok(PinActions {actions: pin_actions}.into_py(py))
    }

    fn drive(&mut self, data: BigUint) -> PyResult<()> {
        let dut = DUT.lock().unwrap();
        self.pin_collection.update(&dut, &Transaction::new_write(data, self.pin_collection.len())?)?;
        Ok(())
    }

    fn verify(&mut self, data: BigUint) -> PyResult<()> {
        let dut = DUT.lock().unwrap();
        self.pin_collection.update(&dut, &Transaction::new_verify(data, self.pin_collection.len())?)?;
        Ok(())
    }

    fn capture(&mut self) -> PyResult<()> {
        let dut = DUT.lock().unwrap();
        self.pin_collection.update(&dut, &Transaction::new_capture(self.pin_collection.len())?)?;
        Ok(())
    }

    fn highz(&mut self) -> PyResult<()> {
        let dut = DUT.lock().unwrap();
        self.pin_collection.update(&dut, &Transaction::new_highz(self.pin_collection.len())?)?;
        Ok(())
    }

    fn reset(&mut self) -> PyResult<()> {
        let dut = DUT.lock().unwrap();
        self.pin_collection.reset(&dut)?;
        Ok(())
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        Ok(self.pin_collection.pin_names(&dut)?)
    }

    #[getter]
    fn get_width(&self) -> PyResult<usize> {
        Ok(self.pin_collection.len())
    }

    #[getter]
    fn get_reset_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pin_actions = self.pin_collection.get_reset_actions(&dut)?;
        Ok(PinActions {actions: pin_actions}.into_py(py))
    }

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        Ok(!self.pin_collection.is_little_endian())
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        Ok(self.pin_collection.is_little_endian())
    }

    #[args(kwargs = "**")]
    fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        locals.set_item("kwargs", kwargs.to_object(py))?;

        py.eval(&format!("origen.tester.cycle(**(kwargs or {{}}))"), None, Some(&locals))?;
        Ok(slf.into())
    }

    fn repeat(slf: PyRef<Self>, count: usize) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        py.eval(&format!("origen.tester.repeat({})", count), None, Some(&locals))?;
        Ok(slf.into())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinCollection {
    // Need to overwrite contains to account for aliasing
    fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
        if let Ok(s) = item.extract::<String>() {
            // For just a String, assume model_id is 0 (DUT-level)
            let dut = DUT.lock().unwrap();
            Ok(self.pin_collection.contains_identifier(&dut, (0, s))?)
        } else {
            Ok(false)
        }
    }
}

impl ListLikeAPI for PinCollection {
    fn item_ids(&self, _dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
        self.pin_collection.pin_ids.clone()
    }

    // Grabs a single pin and puts it in an anonymous pin collection
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        Ok(Py::new(py, PinCollection { 
            pin_collection: OrigenPinCollection::new(
                vec![self.pin_collection.pin_ids[idx]],
                None
            ),
        })?.to_object(py))
    }

    fn __iter__(&self) -> PyResult<ListLikeIter> {
        Ok(ListLikeIter {
            parent: Box::new((*self).clone()),
            i: 0,
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut ids: Vec<usize> = vec![];
        {
            let indices = slice.indices((self.pin_collection.pin_ids.len() as i32).into())?;
            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    ids.push(self.pin_collection.pin_ids[i as usize].clone());
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    ids.push(self.pin_collection.pin_ids[i as usize].clone());
                    i += indices.step;
                }
            }
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinCollection { 
            pin_collection: OrigenPinCollection::new(ids, None),
        })?.to_object(py))
    }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for PinCollection {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        ListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        ListLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinCollection {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
        ListLikeAPI::__iter__(&*slf)
    }
}

impl From<OrigenPinCollection> for PinCollection {
    fn from(collection: OrigenPinCollection) -> Self {
        PinCollection {
            pin_collection: collection,
        }
    }
}
