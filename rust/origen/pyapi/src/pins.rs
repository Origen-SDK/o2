use crate::dut::PyDUT;
use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};
use pyo3::class::mapping::*;

#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pymodule]
/// Implements the module _origen.model in Python
pub fn pins(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pin>()?;
    m.add_class::<PinContainer>()?;
    m.add_class::<PinGroup>()?;
    m.add_class::<PinGroupContainer>()?;
    m.add_class::<PinCollection>()?;
    Ok(())
}

#[pymethods]
impl PyDUT {
    fn add_pin(&self, path: &str, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;
        model.pin_container.add_pin(name, path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, Pin {
                    name: String::from(name),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    fn pin(&self, path: &str, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, Pin {
                    name: String::from(name),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    #[args(aliases = "*")]
    fn add_pin_alias(&self, path: &str, name: &str, aliases: &PyTuple) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        for alias in aliases {
            let _alias: String = alias.extract()?;
            model.pin_container.add_pin_alias(name, &_alias)?;
        }
        Ok(())
    }

    fn pins(&self, path: &str) -> PyResult<Py<PinContainer>> {
        // Even though we won't use the model, make sure the DUT exists and the path is reachable.
        let mut dut = DUT.lock().unwrap();
        let _model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinContainer {path: String::from(path)}).unwrap())
    }

    #[args(pins = "*")]
    fn group_pins(&self, path: &str, name: &str, pins: &PyTuple) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        model.pin_container.group_pins(name, path, pins.extract()?)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin_group(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    name: String::from(name),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    fn pin_group(&self, path: &str, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin_group(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    name: String::from(name),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    //#[args(aliases = "*")]
    //fn add_pin_group_alias(&self, path: &str, name: &str, aliases: &PyTuple) -> PyResult<()> {
    //    Ok(())
    //}

    fn pin_groups(&self, path: &str) ->PyResult<Py<PinGroupContainer>> {
        // even though we won't use the model, make sure the DUT exists and the path is reachable.
        let mut dut = DUT.lock().unwrap();
        let _model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinGroupContainer {path: String::from(path)}).unwrap())
    }
}

#[pyclass]
struct PinContainer {
    path: String,
}

#[pymethods]
impl PinContainer {
    fn keys(&self) -> PyResult<Vec<String>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let names = &model.pin_container.pins;

        let mut v: Vec<String> = Vec::new();
        for (n, _p) in names {
            v.push(n.clone());
        }
        Ok(v)
    }

    fn values(&self) -> PyResult<Vec<Py<Pin>>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pins = &model.pin_container.pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<Py<Pin>> = Vec::new();
        for (n, _p) in pins {
            v.push(Py::new(py, Pin {
                name: String::from(n.clone()),
                path: String::from(self.path.clone()),
            }).unwrap())
        }
        Ok(v)
    } 

    fn items(&self) -> PyResult<Vec<(String, Py<Pin>)>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pins = &model.pin_container.pins;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut items: Vec<(String, Py<Pin>)> = Vec::new();
        for (n, _p) in pins {
            items.push((
                n.clone(),
                Py::new(py, Pin {
                    name: String::from(n.clone()),
                    path: String::from(self.path.clone()),
                }).unwrap(),
            ));
        }
        Ok(items)
    }
}

#[pyproto]
impl PyMappingProtocol for PinContainer {
    fn __getitem__(&self, name: &str) -> PyResult<Py<Pin>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, Pin {
                    name: String::from(name),
                    path: String::from(&self.path),
                }).unwrap())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin or pin alias found for {}", name)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.pin_container.number_of_pins())
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinContainer {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinContainerIter> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&slf.path)?;
        let pin_container = &model.pin_container;
        Ok(PinContainerIter {
            keys: pin_container.pins.iter().map(|(s, _)| s.clone()).collect(),
            i: 0,
        })
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinContainer {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.pin_container.has_pin(item))
    }
}

#[pyclass]
struct PinContainerIter {
    keys: Vec<String>,
    i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinContainerIter {

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    /// The Iterator will be created with an index starting at 0 and the pin names at the time of its creation.
    /// For each call to 'next', we'll create a pin object with the next value in the list, or None, if no more keys are available.
    /// Note: this means that the iterator can become stale if the PinContainer is changed. This can happen if the iterator is stored from Python code
    ///  directly. E.g.: i = dut.pins.__iter__() => iterator with the pin names at the time of creation,
    /// Todo: Fix the above using iterators. My Rust skills aren't there yet though... - Coreyeng
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None)
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}

#[pyclass]
struct Pin {
    name: String,
    path: String,
}

#[pymethods]
impl Pin {

    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);

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
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);
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
        let pin = model.pin_container.pin(&self.name);
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

#[pyclass]
struct PinGroupContainer {
    path: String,
}

#[pyproto]
impl PyMappingProtocol for PinGroupContainer {
    fn __getitem__(&self, name: &str) -> PyResult<Py<PinGroup>> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin_container.pin_group(name);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    name: String::from(name),
                    path: String::from(&self.path),
                }).unwrap())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", name)))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        Ok(model.pin_container.number_of_pin_groups())
    }
}

#[pyclass]
struct PinGroup {
    name: String,
    path: String,
}

#[pymethods]
impl PinGroup {
    //fn drive(&self, path: &str) {}
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    fn __len__(&self) -> PyResult<usize> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        let grp = model.pin_container.pin_group(&self.name);
        match grp {
            Some(_grp) => {
                Ok(_grp.len())
            },
            // Stay in sync with Python's Hash - Raise a KeyError if no pin is found.
            None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.name)))
        }
    }
}

#[pyclass]
struct PinCollection {}

#[pymethods]
impl PinCollection {
    fn drive(&self, _path: &str) {}
}
