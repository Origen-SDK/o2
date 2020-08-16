use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use origen::core::model::pins::pin_collection::PinCollection as OrigenPinCollection;
use origen::core::model::pins::Endianness;
use origen::error::Error;
use origen::{dut, DUT};
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};
use super::pin_actions::PinActions;

#[pyclass]
#[derive(Clone)]
pub struct PinCollection {
    model_id: usize,
    pin_collection: OrigenPinCollection,
}

impl PinCollection {
    pub fn new(
        model_id: usize,
        names: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<PinCollection, Error> {
        let mut dut = dut();
        let collection = dut.collect(model_id, names, endianness)?;
        Ok(PinCollection {
            pin_collection: collection,
            model_id: model_id,
        })
    }
}

#[pymethods]
impl PinCollection {
    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_data(self.pin_collection.model_id, &self.pin_collection.pin_names)?)
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_collection_data(&self.pin_collection, data)?;
        Ok(())

        // let gil = Python::acquire_gil();
        // let py = gil.python();

        // // I'm sure there's a better way to return self, but I wasn't able to get anything to work.
        // // Just copying self and returning that for now.
        // Ok(Py::new(
        //     py,
        //     PinCollection {
        //         pin_collection: self.pin_collection.clone(),
        //         model_id: self.model_id,
        //     },
        // )
        // .unwrap())
    }

    fn with_mask(&mut self, mask: usize) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_collection_nonsticky_mask(&mut self.pin_collection, mask)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            PinCollection {
                pin_collection: self.pin_collection.clone(),
                model_id: self.model_id,
            },
        )
        .unwrap())
    }

    fn set(&self, data: u32) -> PyResult<Py<Self>> {
        self.set_data(data)?;

        let gil = Python::acquire_gil();
        let py = gil.python();

        // I'm sure there's a better way to return self, but I wasn't able to get anything to work.
        // Just copying self and returning that for now.
        Ok(Py::new(
            py,
            PinCollection {
                pin_collection: self.pin_collection.clone(),
                model_id: self.model_id,
            },
        )
        .unwrap())
    }

    #[setter]
    fn pin_actions(&mut self, actions: &PyAny) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.set_per_pin_collection_actions(
            &mut self.pin_collection,
            &extract_pinactions!(actions)?
        )?;
        Ok(())
    }

    fn set_actions(mut slf: PyRefMut<Self>, actions: &PyAny) -> PyResult<Py<Self>> {
        slf.pin_actions(actions)?;
        Ok(slf.into())
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let pin_actions = dut.get_pin_actions(self.model_id, &self.pin_collection.pin_names)?;
        Ok(PinActions {actions: pin_actions}.into_py(py))
    }

    fn drive(&mut self, data: Option<u32>) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.drive_pin_collection(&mut self.pin_collection, data)?;
        Ok(())
    }

    fn verify(&mut self, data: Option<u32>) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.verify_pin_collection(&mut self.pin_collection, data)?;
        Ok(())
    }

    fn capture(&mut self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.capture_pin_collection(&mut self.pin_collection)?;
        Ok(())
    }

    fn highz(&mut self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.highz_pin_collection(&mut self.pin_collection)?;
        Ok(())
    }

    fn reset(&mut self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.reset_pin_collection(&mut self.pin_collection)?;
        Ok(())
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        Ok(self.pin_collection.pin_names.clone())
    }

    #[getter]
    fn get_width(&self) -> PyResult<usize> {
        Ok(self.pin_collection.len())
    }

    #[getter]
    fn get_reset_data(&self) -> PyResult<u32> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_collection_reset_data(&self.pin_collection)?)
    }

    #[getter]
    fn get_reset_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pin_actions = dut.get_pin_collection_reset_actions(&self.pin_collection)?;
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
            let dut = DUT.lock().unwrap();
            Ok(dut.pin_names_contain(self.model_id, &self.pin_collection.pin_names, &s)?)
        } else {
            Ok(false)
        }
    }
}

impl ListLikeAPI for PinCollection {
    fn item_ids(&self, dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
        let mut pin_ids: Vec<usize> = vec![];
        for pname in self.pin_collection.pin_names.iter() {
            pin_ids.push(dut._get_pin(self.model_id, pname).unwrap().id);
        }
        pin_ids
    }

    // Grabs a single pin and puts it in an anonymous pin collection
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        Ok(Py::new(
            py,
            PinCollection::new(
                self.model_id,
                vec![self.pin_collection.pin_names[idx].clone()],
                None,
            )?,
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

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut names: Vec<String> = vec![];
        {
            let indices = slice.indices((self.pin_collection.pin_names.len() as i32).into())?;
            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    names.push(self.pin_collection.pin_names[i as usize].clone());
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    names.push(self.pin_collection.pin_names[i as usize].clone());
                    i += indices.step;
                }
            }
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinCollection::new(self.model_id, names, None)?)
            .unwrap()
            .to_object(py))
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
            model_id: collection.model_id.clone(),
            pin_collection: collection,
        }
    }
}
