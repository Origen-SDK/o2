use origen::{DUT, lock};
use origen::error::Error;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes, PySlice};
use origen::core::model::pins::pin_collection::PinCollection as OrigenPinCollection;

#[pyclass]
pub struct PinCollection {
    path: String,
    pin_collection: OrigenPinCollection
}

impl PinCollection{
  pub fn new(path: &str, ids: Vec<String>) -> Result<PinCollection, Error> {
    let mut dut = lock!()?;
    let model = dut.get_mut_model(path)?;
    let collection = model.collect(path, ids)?;
    Ok(PinCollection {
        path: String::from(path),
        pin_collection: collection,
    })
  }
}

#[pymethods]
impl PinCollection {
    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.pin_collection.path)?;
        Ok(model.get_pin_data(&self.pin_collection.ids))
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        model.set_pin_data(&self.pin_collection.ids, data)?;
        Ok(())
    }

    fn set(&self, data: u32) -> PyResult<()> {
        return self.set_data(data);
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.get_pin_actions(&self.pin_collection.ids)?)
    }

    fn drive(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.drive_pins(&self.pin_collection.ids, data)?;
      Ok(())
    }

    fn verify(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.verify_pins(&self.pin_collection.ids, data)?;
      Ok(())
    }

    fn capture(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.capture_pins(&self.pin_collection.ids)?;
      Ok(())
    }

    fn highz(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.highz_pins(&self.pin_collection.ids)?;
      Ok(())
    }

    #[getter]
    fn get_ids(&self) -> PyResult<Vec<String>> {
        Ok(self.pin_collection.ids.clone())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get__path(&self) -> PyResult<String> {
        Ok(self.pin_collection.path.clone())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinCollection {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.pin_collection.len())
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.pin_ids_contain(&self.pin_collection.ids, item)?)
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
          let collection = self.pin_collection.slice_ids(indices.start as usize, indices.stop as usize, indices.step as usize)?;
          Ok(Py::new(py, PinCollection::from(collection)).unwrap().to_object(py))
        } else {
          let i = idx.extract::<isize>().unwrap();
          let collection = self.pin_collection.slice_ids(i as usize, i as usize, 1)?;
          Ok(Py::new(py, PinCollection::from(collection)).unwrap().to_object(py))
        }
      }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinCollection {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinCollectionIter> {
        Ok(PinCollectionIter {
            keys: slf.pin_collection.ids.clone(),
            i: 0,
            path: slf.path.clone(),
        })
    }
}

impl From<OrigenPinCollection> for PinCollection {
  fn from(collection: OrigenPinCollection) -> Self {
    PinCollection {
      path: collection.path.clone(),
      pin_collection: collection,
    }
  }
}

#[pyclass]
pub struct PinCollectionIter {
  keys: Vec<String>,
  i: usize,
  path: String,
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
        return Ok(None)
    }
    let id = slf.keys[slf.i].clone();
    let collection = PinCollection::new(&(slf.path.as_str()), vec!(id))?;
    slf.i += 1;
    Ok(Some(collection))
  }
}