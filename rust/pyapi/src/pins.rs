use crate::dut::PyDUT;
use crate::get_full_class_name;
use crate::unpack_transaction_kwargs;
use origen::DUT;
use pyo3::prelude::*;

#[macro_use]
pub mod pin_actions;
#[macro_use]
pub mod pin;
#[macro_use]
mod pin_group;
#[macro_use]
mod pin_collection;
mod physical_pin_container;
mod pin_container;
pub mod pin_header;

use origen::core::model::pins::Endianness;
use physical_pin_container::PhysicalPinContainer;
use pin::Pin;
use pin_actions::PinActions;
use pin_collection::PinCollection;
use pin_container::PinContainer;
use pin_group::PinGroup;
use pin_header::{PinHeader, PinHeaderContainer};

#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PyTuple};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "pins")?;
    subm.add_class::<Pin>()?;
    subm.add_class::<PinContainer>()?;
    subm.add_class::<PinGroup>()?;
    subm.add_class::<PinCollection>()?;
    subm.add_class::<PinHeader>()?;
    subm.add_class::<PinHeaderContainer>()?;
    subm.add_class::<PinActions>()?;

    pyapi_metal::alias_method_apply_to_set!(subm, "PinGroup", "actions");
    pyapi_metal::alias_method_apply_to_set!(subm, "PinCollection", "actions");
    m.add_submodule(subm)?;
    Ok(())
}

pub fn extract_pin_transaction(
    actions: &PyAny,
    action: origen::TransactionAction,
    kwargs: Option<&PyDict>,
) -> PyResult<origen::Transaction> {
    let actions = extract_pinactions!(actions)?;
    let mut t;
    match action {
        origen::TransactionAction::Set => t = origen::Transaction::new_set(&actions)?,
        _ => {
            return crate::runtime_error!(format!(
                "Extracting Pin transaction of action {:?} is not supported",
                action
            ))
        }
    }
    unpack_pin_transaction_kwargs(&mut t, kwargs)?;
    Ok(t)

    // if let Some(opts) = kwargs {
    //     if let Some(mask) = opts.get_item("mask") {
    //         if let Ok(big_mask) = mask.extract::<num_bigint::BigUint>() {
    //             t.bit_enable = big_mask;
    //         } else {
    //             return super::type_error!("Could not extract kwarg 'mask' as an integer");
    //         }
    //     }
    //     if let Some(overlay) = opts.get_item("overlay") {
    //         let overlay_mask;
    //         if let Some(mask) = opts.get_item("overlay_mask") {
    //             if let Ok(big_mask) = mask.extract::<num_bigint::BigUint>() {
    //                 overlay_mask = Some(big_mask);
    //             } else {
    //                 return super::type_error!("Could not extract kwarg 'overlay_mask' as an integer");
    //             }
    //         } else {
    //             overlay_mask = Some(t.enable_width()?)
    //         }
    //         // panic!("option not supported yet!");
    //         if let Ok(should_overlay) = overlay.extract::<bool>() {
    //             if should_overlay {
    //                 // Unnamed overlay
    //                 t.overlay_enable = overlay_mask;
    //             }
    //         } else if let Ok(overlay_name) = overlay.extract::<String>() {
    //             t.overlay_enable = overlay_mask;
    //             t.overlay_string = Some(overlay_name);
    //         } else {
    //             return super::type_error!("Could not extract kwarg 'overlay' as either a bool or a string");
    //         }
    //     }

    // if let Some(_overlay_str) = opts.get_item("overlay_str") {
    //     panic!("option not supported yet!");
    // }
    // }
    // Ok(t)
}

fn unpack_pin_transaction_kwargs(
    trans: &mut origen::Transaction,
    kwargs: Option<&PyDict>,
) -> PyResult<()> {
    if let Some(opts) = kwargs {
        unpack_transaction_kwargs(trans, opts)?;
    }
    Ok(())
}

/// Given a vector of PyAny's, assumed to be either a String, Pin, or PinGroup object,
/// return a vector with each item mapped as (model_id<usize>, name<String>) pairs.
/// This pair is sufficient to lookup the pin group object in the backend.
///  - This doesn't resolve any pin groups.
///  - If a String is given, its model_id is assumed to be 0 (on the DUT).
pub fn pins_to_backend_lookup_fields(
    py: Python,
    pins: &PyTuple,
) -> Result<Vec<(usize, String)>, PyErr> {
    let mut retn: Vec<(usize, String)> = vec![];
    for (i, p) in pins.iter().enumerate() {
        if let Ok(s) = p.extract::<String>() {
            // item is a String (or extract-able as a String)
            // Model ID is 0.
            retn.push((0, s.clone()));
        } else if p.get_type().name()? == "Pin" || p.get_type().name()? == "PinGroup" {
            let obj = p.to_object(py);
            let model_id = obj
                .getattr(py, "__origen__model_id__")?
                .extract::<usize>(py)?;
            let name = obj.getattr(py, "name")?.extract::<String>(py)?;
            retn.push((model_id, name.to_string()));
        } else {
            return Err(PyErr::from(origen::Error::new(&format!(
                "Could not resolve object at index {} as String, Pin, or Pin Group. Got: {}",
                i,
                p.get_type().name()?
            ))));
        }
    }
    Ok(retn)
}

pub fn vec_to_ppin_ids(dut: &origen::Dut, pins: Vec<&PyAny>) -> PyResult<Vec<usize>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = PyTuple::new(py, pins);
    let pin_lookups = pins_to_backend_lookup_fields(py, t)?;
    let flat_pins = dut._resolve_to_flattened_pins(&pin_lookups)?;
    Ok(flat_pins.iter().map(|p| p.id).collect::<Vec<usize>>())
}

// /// Similar to pins_to_backend_lookup_fields (and actually uses that) but goes
// /// the extra step of converting the pins to actual pin_ids
// /// This will catch pin-related errors sooner, but requires that the pins
// /// are already created.
// pub fn extract_as_pin_ids(dut: Dut, pins: &PyTuple) -> PyResult<Vec<usize>> {
//     let lookup_fields = pins_to_backend_lookup_fields(py, pins)?;
//     let retn: Vec<usize> = vec![];
//     for (model_id, name) in lookup_fields {
//         // ...
//     }
//     Ok(retn)
// }

#[pymethods]
impl PyDUT {
    #[args(kwargs = "**")]
    fn add_pin(&self, model_id: usize, name: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let (mut reset_action, mut width, mut offset, mut endianness): (
            Option<Vec<origen::core::model::pins::pin::PinAction>>,
            Option<u32>,
            Option<u32>,
            Option<Endianness>,
        ) = (Option::None, Option::None, Option::None, Option::None);
        match kwargs {
            Some(args) => {
                if let Some(arg) = args.get_item("reset_action") {
                    reset_action = Some(extract_pinactions!(arg)?);
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
        dut.add_pin(model_id, name, width, offset, reset_action, endianness)?;

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
        Ok(Py::new(py, PinContainer { model_id: model_id }).unwrap())
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
        let mut name_strs: Vec<String> = vec![];
        for (_i, n) in pins.iter().enumerate() {
            let cls = get_full_class_name(n)?;
            if cls == "re.Pattern" || cls == "_sre.SRE_Pattern" {
                let r = n.getattr("pattern").unwrap();
                name_strs.push(format!("/{}/", r));
            } else {
                let _n = n.extract::<String>()?;
                name_strs.push(_n.clone());
            }
        }
        let mut dut = DUT.lock().unwrap();
        dut.group_pins_by_name(model_id, name, name_strs, endianness)?;

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
        Ok(Py::new(py, PhysicalPinContainer { model_id: model_id }).unwrap())
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

    #[args(pins = "*")]
    fn add_pin_header(
        &self,
        model_id: usize,
        name: &str,
        pins: &PyTuple,
    ) -> PyResult<Py<PinHeader>> {
        let mut dut = DUT.lock().unwrap();
        dut.create_pin_header(model_id, name, pins.extract::<Vec<String>>()?)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            PinHeader {
                name: String::from(name),
                model_id: model_id,
            },
        )
        .unwrap())
    }

    fn pin_headers(&self, model_id: usize) -> PyResult<Py<PinHeaderContainer>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinHeaderContainer { model_id: model_id }).unwrap())
    }

    fn pin_header(&self, model_id: usize, name: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        match dut.get_pin_header(model_id, name) {
            Some(_p) => Ok(Py::new(
                py,
                PinHeader {
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
