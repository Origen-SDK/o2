use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};
use super::pin::Pin;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes, PySlice};

#[pyclass]
pub struct PinGroup {
    pub id: String,
    pub path: String,
}

#[pymethods]
impl PinGroup {
    // Even though we're storing the id in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_id(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id);
        match grp {
            Some(_grp) => {
                Ok(_grp.id.clone())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.get_pin_group_data(&self.id))
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        model.set_pin_group_data(&self.id, data)?;
        Ok(())
    }

    fn set(&self, data: u32) -> PyResult<()> {
        return self.set_data(data);
    }

    #[getter]
    fn get_pin_ids(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id);

        let mut v: Vec<String> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_ids.iter() {
                    v.push(n.clone());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_pins(&self) -> PyResult<Vec<Py<Pin>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<Pin>> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_ids.iter() {
                    v.push(Py::new(py, Pin {
                        id: String::from(n),
                        path: String::from(&self.path),
                    }).unwrap());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.get_pin_actions_for_group(&self.id))
    }

    fn drive(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.drive_pin_group(&self.id, Option::None, data)?)
    }

    fn verify(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.verify_pin_group(&self.id, Option::None, data)?)
    }

    fn capture(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.capture_pin_group(&self.id, Option::None)?)
    }

    fn highz(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.highz_pin_group(&self.id, Option::None)?)
    }

    // Debug helper: Get the id held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__id(&self) -> PyResult<String> {
        Ok(self.id.clone())
    }

    // Debug helper: Get the id held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__path(&self) -> PyResult<String> {
        Ok(self.path.clone())
    }

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        let is_little_endian = self.get_little_endian()?;
        Ok(!is_little_endian)
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id);

        match grp {
            Some(_grp) => {
                Ok(_grp.is_little_endian())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.id))))
            }
        }
    }

}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id);
        match grp {
            Some(_grp) => {
                Ok(_grp.len())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.id)))
        }
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.pin_group_contains_pin(&self.id, item))
    }
}

#[pyproto]
impl<'p> pyo3::class::PyMappingProtocol<'p> for PinGroup {
    // Indexing example: https://github.com/PyO3/pyo3/blob/master/tests/test_dunder.rs#L423-L438
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.id).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Ok(slice) = idx.cast_as::<PySlice>() {
          let mut v: Vec<String> = vec!();

          // Indices requires (what I think is) a max size. Limiting to 1024 right now. Should be plenty.
          let indices = slice.indices(1024)?;
          for i in (indices.start..=indices.stop).step_by(indices.step as usize) {
            let p = grp.pin_ids[i as usize].clone();
            v.push(p);
          }
          Ok(v.to_object(py))
        } else {
          let i = idx.extract::<isize>().unwrap();
          Ok(grp.pin_ids[i as usize].clone().to_object(py))
        }
    }
}