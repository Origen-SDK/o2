use crate::dut::PyDUT;
use origen::DUT;
use pyo3::prelude::*;

#[macro_use]
mod pin;
#[macro_use]
mod pin_group;
#[macro_use]
mod pin_collection;
mod physical_pin_container;
mod pin_container;

use origen::core::model::pins::Endianness;
use physical_pin_container::PhysicalPinContainer;
use pin::Pin;
use pin_collection::PinCollection;
use pin_container::PinContainer;
use pin_group::PinGroup;

#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PyTuple};

#[pymodule]
/// Implements the module _origen.pins in Python
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
    fn add_pin(&self, model_id: usize, name: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let (mut reset_data, mut reset_action, mut width, mut offset, mut endianness): (
            Option<u32>,
            Option<String>,
            Option<u32>,
            Option<u32>,
            Option<Endianness>,
        ) = (
            Option::None,
            Option::None,
            Option::None,
            Option::None,
            Option::None,
        );
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
            }
            None => {}
        }
        dut.add_pin(
            model_id,
            name,
            width,
            offset,
            reset_data,
            reset_action,
            endianness,
        )?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match dut.get_pin_group(model_id, name) {
            Some(_p) => Ok(Py::new(
                py,
                PinGroup {
                    name: String::from(name),
                    model_id: model_id,
                },
            )
            .unwrap()
            .to_object(py)),
            None => Ok(py.None()),
        }
    }

    fn pin(&self, model_id: usize, name: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        match dut.get_pin_group(model_id, name) {
            Some(_p) => Ok(Py::new(
                py,
                PinGroup {
                    name: String::from(name),
                    model_id: model_id,
                },
            )
            .unwrap()
            .to_object(py)),
            None => Ok(py.None()),
        }
    }

    #[args(aliases = "*")]
    fn add_pin_alias(&self, model_id: usize, name: &str, aliases: &PyTuple) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        for alias in aliases {
            let _alias: String = alias.extract()?;
            dut.add_pin_alias(model_id, name, &_alias)?;
        }
        Ok(())
    }

    fn pins(&self, model_id: usize) -> PyResult<Py<PinContainer>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            PinContainer {
                model_id: model_id,
            },
        )
        .unwrap())
    }

    #[args(pins = "*", options = "**")]
    fn group_pins(
        &self,
        model_id: usize,
        name: &str,
        pins: &PyTuple,
        options: Option<&PyDict>,
    ) -> PyResult<PyObject> {
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
            }
            None => {}
        }
        let mut dut = DUT.lock().unwrap();
        dut.group_pins_by_name(model_id, name, pins.extract()?, endianness)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            PinGroup {
                name: String::from(name),
                model_id: model_id,
            },
        )
        .unwrap()
        .to_object(py))
    }

    fn physical_pins(&self, model_id: usize) -> PyResult<Py<PhysicalPinContainer>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            PhysicalPinContainer {
                model_id: model_id,
            },
        )
        .unwrap())
    }

    fn physical_pin(&self, model_id: usize, name: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        match dut.get_pin(model_id, name) {
            Some(_p) => Ok(Py::new(
                py,
                Pin {
                    name: String::from(name),
                    model_id: model_id,
                },
            )
            .unwrap()
            .to_object(py)),
            None => Ok(py.None()),
        }
    }
}
