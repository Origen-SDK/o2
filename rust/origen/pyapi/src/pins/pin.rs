use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
use pyo3::{PyErr};
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pyclass]
pub struct Pin {
    pub name: String,
    pub path: String,
}

#[pymethods]
impl Pin {

    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        match pin {
            Some(_pin) => {
                Ok(_pin.name.clone())
            },
            Option::None => {
                // This is problem, since we should only have a Pin instance if the pin exists. This would be a stale instance.
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }

    #[getter]
    fn get_data(&self) -> PyResult<u8> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);

        match pin {
            Some(_pin) => {
                Ok(_pin.data)
            },
            Option::None => {
                // This is problem, since we should only have a Pin instance if the pin exists. This would be a stale instance.
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }

    #[setter]
    fn set_data(&self, data: u8) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        match pin {
            Some(_pin) => {
                _pin.set_data(data)?;
                Ok(())
            }
            Option::None => {
                // This is problem, since we should only have a Pin instance if the pin exists. This would be a stale instance.
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }

    fn set(&self, data: u8) -> PyResult<()> {
        self.set_data(data)
    }

    #[getter]
    fn get_action(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        match pin {
            Some(_pin) => {
                Ok(String::from(_pin.action.as_str()))
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }

    // #[getter]
    // fn state(&self, path: &str) -> PyResult<PyString> {}

    fn drive(&mut self, data: Option<u8>) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        //let pin = pin!();
        //let mut pin = pin!(self)?;
        match pin {
            Some(_pin) => {
                _pin.drive(data)?;
                Ok(())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }

    fn verify(&self, data: Option<u8>) -> PyResult<()>  { 
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        //let pin = pin!();
        //let mut pin = pin!(self)?;
        match pin {
            Some(_pin) => {
                _pin.verify(data)?;
                Ok(())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }
    
    fn capture(&self) -> PyResult<()>  {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        //let pin = pin!();
        //let mut pin = pin!(self)?;
        match pin {
            Some(_pin) => {
                _pin.capture()?;
                Ok(())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
    }
    
    fn highz(&self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin(&self.name);
        match pin {
            Some(_pin) => {
                _pin.highz()?;
                Ok(())
            },
            Option::None => {
                Err(PyErr::from(Error::new(&format!("Stale reference to pin {}", self.name))))
            }
        }
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
}