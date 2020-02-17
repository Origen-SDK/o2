use origen::core::model::pins::pin_collection::PinCollection as OrigenPinCollection;
use origen::core::model::pins::Endianness;
use origen::error::Error;
use origen::{lock, DUT};
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};

#[pyclass]
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
        let mut dut = lock!()?;
        //let model = dut.get_mut_model(model_id)?;
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
    fn set_data(&self, data: u32) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_collection_data(&self.pin_collection, data)?;

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
        return self.set_data(data);
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_actions(self.model_id, &self.pin_collection.pin_names)?)
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
    fn get_reset_actions(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_collection_reset_actions(&self.pin_collection)?)
    }

    // #[allow(non_snake_case)]
    // #[getter]
    // fn get__path(&self) -> PyResult<String> {
    //     Ok(self.pin_collection.path.clone())
    // }

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        Ok(!self.pin_collection.is_little_endian())
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        Ok(self.pin_collection.is_little_endian())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinCollection {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.pin_collection.len())
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let dut = DUT.lock().unwrap();
        Ok(dut.pin_names_contain(self.model_id, &self.pin_collection.pin_names, item)?)
    }
}

#[pyproto]
impl<'p> pyo3::class::PyMappingProtocol<'p> for PinCollection {
    // Indexing example: https://github.com/PyO3/pyo3/blob/master/tests/test_dunder.rs#L423-L438
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            // Indices requires (what I think is) a max size. Should be plenty.
            let indices = slice.indices(8192)?;
            let collection = self.pin_collection.slice_names(
                indices.start as usize,
                indices.stop as usize,
                indices.step as usize,
            )?;
            Ok(Py::new(py, PinCollection::from(collection))
                .unwrap()
                .to_object(py))
        } else {
            let i = idx.extract::<isize>().unwrap();
            let collection = self.pin_collection.slice_names(i as usize, i as usize, 1)?;
            Ok(Py::new(py, PinCollection::from(collection))
                .unwrap()
                .to_object(py))
        }
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinCollection {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinCollectionIter> {
        Ok(PinCollectionIter {
            keys: slf.pin_collection.pin_names.clone(),
            i: 0,
            model_id: slf.model_id,
        })
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

#[pyclass]
pub struct PinCollectionIter {
    keys: Vec<String>,
    i: usize,
    model_id: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinCollectionIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PinCollection>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        let collection = PinCollection::new(slf.model_id, vec![name], Option::None)?;
        slf.i += 1;
        Ok(Some(collection))
    }
}
