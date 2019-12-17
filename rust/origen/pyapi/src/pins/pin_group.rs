use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};
use super::pin::Pin;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pyclass]
pub struct PinGroup {
    pub name: String,
    pub path: String,
}

#[pymethods]
impl PinGroup {
    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.name);
        match grp {
            Some(_grp) => {
                Ok(_grp.name.clone())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.name))))
            }
        }
    }

    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.get_pin_group_data(&self.name))
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        model.set_pin_group_data(&self.name, data)?;
        Ok(())
    }

    fn set(&self, data: u32) -> PyResult<()> {
        return self.set_data(data);
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.name);

        let mut v: Vec<String> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_names.iter() {
                    v.push(n.clone());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.name))))
            }
        }
    }

    #[getter]
    fn get_pins(&self) -> PyResult<Vec<Py<Pin>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.name);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<Pin>> = Vec::new();
        match grp {
            Some(_grp) => {
                for n in _grp.pin_names.iter() {
                    v.push(Py::new(py, Pin {
                        name: String::from(n),
                        path: String::from(&self.path),
                    }).unwrap());
                }
                Ok(v)
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.name))))
            }
        }
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.get_pin_actions_for_group(&self.name))
    }

    // Debug helper: Get the name held by this instance.
    #[allow(non_snake_case)]
    #[getter]
    fn get__name(&self) -> PyResult<String> {
        Ok(self.name.clone())
    }

    // Debug helper: Get the name held by this instance.
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
        let grp = model.pin_group(&self.name);

        match grp {
            Some(_grp) => {
                Ok(_grp.is_little_endian())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin group {}", self.name))))
            }
        }
    }

}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_group(&self.name);
        match grp {
            Some(_grp) => {
                Ok(_grp.len())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.name)))
        }
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.pin_group_contains_pin(&self.name, item))
        // let grp = model.pin_group(&self.name);
        // match grp {
        //     Some(_grp) => {
        //         Ok(_grp.contains_pin(model, item))
        //         //Ok(true)
        //     },
        //     None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.name)))
        // }
        //Ok(true)
    }
}