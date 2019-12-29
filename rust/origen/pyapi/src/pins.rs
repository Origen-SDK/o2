use crate::dut::PyDUT;
use origen::DUT;
use pyo3::prelude::*;

#[macro_use] mod pin;
#[macro_use] mod pin_group;
#[macro_use] mod pin_collection;
mod pin_container;
mod physical_pin_container;

use pin::{Pin};
use pin_group::PinGroup;
use pin_container::PinContainer;
use pin_collection::PinCollection;
use physical_pin_container::PhysicalPinContainer;
use std::collections::HashMap;
use origen::core::model::pins::pin::PinActions;
use origen::core::model::pins::Endianness;

#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};

#[pymodule]
/// Implements the module _origen.model in Python
pub fn pins(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pin>()?;
    m.add_class::<PinContainer>()?;
    m.add_class::<PinGroup>()?;
    m.add_class::<PinCollection>()?;
    Ok(())
}

#[pymethods]
impl PyDUT {

    #[args(kwargs = "**")]
    fn add_pin(&self, path: &str, id: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;
        let (mut reset_data, mut reset_action, mut width, mut offset, mut endianness): (Option<u32>, Option<String>, Option<u32>, Option<u32>, Option<Endianness>) = (Option::None, Option::None, Option::None, Option::None, Option::None);
        match kwargs {
            Some(args) => {
                if let Some(arg) = args.get_item("reset_data") {
                    reset_data = Option::Some(arg.extract::<u32>()?);
                }
                if let Some(arg) = args.get_item("reset_action") {
                    reset_action = Option::Some(arg.extract::<String>()?);
                }
                if let Some(arg) = args.get_item("width") {
                    width = Option::Some(arg.extract::<u32>()?);
                }
                if let Some(arg) = args.get_item("offset") {
                    offset = Option::Some(arg.extract::<u32>()?);
                }
                if let Some(arg) = args.get_item("little_endian") {
                    if arg.extract::<bool>()? {
                        endianness = Option::Some(Endianness::LittleEndian);
                    } else {
                        endianness = Option::Some(Endianness::BigEndian);
                    }
                }
            },
            None => {},
        }
        model.add_pin(id, path, width, offset, reset_data, reset_action, endianness)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    id: String::from(id),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    fn pin(&self, path: &str, id: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    id: String::from(id),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    #[args(aliases = "*")]
    fn add_pin_alias(&self, path: &str, id: &str, aliases: &PyTuple) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        for alias in aliases {
            let _alias: String = alias.extract()?;
            model.add_pin_alias(id, &_alias)?;
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

    #[args(pins = "*", options = "**")]
    fn group_pins(&self, path: &str, id: &str, pins: &PyTuple, options: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let mut endianness = Option::None;
        match options {
            Some(opts) => {
                if let Some(opt) = opts.get_item("little_endian") {
                    if opt.extract::<bool>()? {
                        endianness = Option::Some(Endianness::LittleEndian);
                    } else {
                        endianness = Option::Some(Endianness::BigEndian);
                    }
                }
            },
            None => {}
        }
        let model = dut.get_mut_model(path)?;
        model.group_pins(id, path, pins.extract()?, endianness)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, PinGroup {
                    id: String::from(id),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }

    fn physical_pins(&self, path: &str) -> PyResult<Py<PhysicalPinContainer>> {
        // Even though we won't use the model, make sure the DUT exists and the path is reachable.
        let mut dut = DUT.lock().unwrap();
        let _model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PhysicalPinContainer {path: String::from(path)}).unwrap())
    }

    fn physical_pin(&self, path: &str, id: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(path)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let p = model.physical_pin(id);
        match p {
            Some(_p) => {
                Ok(Py::new(py, Pin {
                    id: String::from(id),
                    path: String::from(path),
                }).unwrap().to_object(py))
            },
            None => Ok(py.None())
        }
    }
}
