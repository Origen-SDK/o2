use num_bigint::BigUint;
use origen::core::model::registers::BitCollection as RichBC;
use origen::core::model::registers::Register;
use origen::Dut;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::PyMappingProtocol;
use pyo3::exceptions;
use pyo3::exceptions::AttributeError;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyInt, PySlice, PyString};
use std::iter::FromIterator;
use std::sync::MutexGuard;

import_exception!(origen.errors, UndefinedDataError);

/// A BitCollection represents either a whole register or a subset of a
/// registers bits (not necessarily contiguous bits) and provides the user
/// with the same API to set and consume register data in both cases.
#[pyclass]
#[derive(Debug, Clone)]
pub struct BitCollection {
    /// The ID of the parent register
    pub reg_id: Option<usize>,
    pub field: Option<String>,
    /// When true the BitCollection contains an entire register's worth of bits
    pub whole_reg: bool,
    pub whole_field: bool,
    pub bit_ids: Vec<usize>,
    /// Iterator index
    pub i: usize,
}

/// Rust-private methods, i.e. not accessible from Python
impl BitCollection {
    pub fn from_reg_id(id: usize, dut: &MutexGuard<Dut>) -> BitCollection {
        let reg = dut.get_register(id).unwrap();
        BitCollection {
            reg_id: Some(id),
            field: None,
            whole_reg: true,
            whole_field: false,
            bit_ids: reg.bit_ids.clone(),
            i: 0,
        }
    }
}

#[pyproto]
impl PyObjectProtocol for BitCollection {
    fn __repr__(&self) -> PyResult<String> {
        if self.reg_id.is_some() {
            let dut = origen::dut();
            let reg = dut.get_register(self.reg_id.unwrap())?;
            if self.field.is_none() {
                if self.whole_reg {
                    Ok(reg.console_display(&dut, None, true)?)
                } else {
                    Ok(format!(
                        "<BitCollection: a subset of '{}' ({} bit(s))>",
                        reg.name,
                        self.bit_ids.len(),
                    ))
                }
            } else {
                if self.whole_field {
                    Ok(format!(
                        "<BitCollection: '{}.{}'>",
                        reg.name,
                        self.field.as_ref().unwrap()
                    ))
                } else {
                    Ok(format!(
                        "<BitCollection: a subset of '{}.{}' ({} bits(s))>",
                        reg.name,
                        self.field.as_ref().unwrap(),
                        self.bit_ids.len(),
                    ))
                }
            }
        } else {
            Ok(format!(
                "<BitCollection: an ad-hoc collection of {} bit(s)>",
                self.bit_ids.len()
            ))
        }
    }

    /// Implements my_reg.my_bits
    fn __getattr__(&self, query: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        if self.whole_reg {
            if query == "bits" || query == "fields" {
                Ok(self.clone())
            } else {
                let reg = dut.get_register(self.reg_id.unwrap())?;
                if reg.fields.contains_key(query) {
                    Ok(BitCollection {
                        reg_id: self.reg_id,
                        field: Some(query.to_string()),
                        whole_reg: false,
                        whole_field: true,
                        bit_ids: reg.fields.get(query).unwrap().bit_ids(&dut),
                        i: 0,
                    })
                } else {
                    Err(AttributeError::py_err(format!(
                        "'BitCollection' object has no attribute '{}'",
                        query
                    )))
                }
            }
        } else {
            Err(AttributeError::py_err(format!(
                "'BitCollection' object has no attribute '{}'",
                query
            )))
        }
    }
}

#[pyproto]
impl PyMappingProtocol for BitCollection {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.bit_ids.len())
    }

    /// Implements my_reg[5]
    fn __getitem__(&self, idx: &PyAny) -> PyResult<BitCollection> {
        let field = match &self.field {
            Some(x) => Some(x.to_string()),
            None => None,
        };
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            // Indices requires (what I think is) a max size. Should be plenty.
            let indices = slice.indices(8192)?;
            // TODO: Should this support step size?
            let upper;
            let lower;
            if indices.start > indices.stop {
                upper = indices.start as usize;
                lower = indices.stop as usize;
            } else {
                upper = indices.stop as usize;
                lower = indices.start as usize;
            }
            let bit_ids = Vec::from_iter(self.bit_ids[lower..upper + 1].iter().cloned());
            Ok(BitCollection {
                reg_id: self.reg_id,
                field: field,
                whole_reg: self.whole_reg && self.bit_ids.len() == bit_ids.len(),
                whole_field: self.whole_field && self.bit_ids.len() == bit_ids.len(),
                bit_ids: bit_ids,
                i: 0,
            })
        } else if let Ok(_int) = idx.cast_as::<PyInt>() {
            let i = idx.extract::<usize>().unwrap();
            if i < self.bit_ids.len() {
                let mut bit_ids: Vec<usize> = Vec::new();
                bit_ids.push(self.bit_ids[i]);
                Ok(BitCollection {
                    reg_id: self.reg_id,
                    field: field,
                    whole_reg: self.whole_reg && self.bit_ids.len() == bit_ids.len(),
                    whole_field: self.whole_field && self.bit_ids.len() == bit_ids.len(),
                    bit_ids: bit_ids,
                    i: 0,
                })
            } else {
                Err(PyErr::new::<exceptions::RuntimeError, _>(
                    "The given bit index is out of range",
                ))
            }
        } else if let Ok(_name) = idx.cast_as::<PyString>() {
            if self.whole_reg {
                let name = idx.extract::<&str>().unwrap();
                self.field(name)
            } else {
                Err(PyErr::new::<exceptions::RuntimeError, _>(
                    "Illegal bit index given",
                ))
            }
        } else {
            Err(PyErr::new::<exceptions::RuntimeError, _>(
                "Illegal bit index given",
            ))
        }
    }
}

/// User API methods
#[pymethods]
impl BitCollection {
    fn len(&self) -> PyResult<usize> {
        Ok(self.bit_ids.len())
    }

    fn reset(&self, name: Option<&str>) -> PyResult<BitCollection> {
        let dut = origen::dut();
        match name {
            Some(n) => self.materialize(&dut)?.reset(n, &dut)?,
            None => self.materialize(&dut)?.reset("hard", &dut)?,
        };
        Ok(self.clone())
    }

    #[args(shift_in = "0")]
    fn shift_left(&self, shift_in: u8) -> PyResult<u8> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.shift_left(shift_in)?)
    }

    #[args(shift_in = "0")]
    fn shift_right(&self, shift_in: u8) -> PyResult<u8> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.shift_right(shift_in)?)
    }

    /// An alias for get_data()
    fn data(&self) -> PyResult<BigUint> {
        self.get_data()
    }

    fn get_data(&self) -> PyResult<BigUint> {
        let dut = origen::dut();
        match self.materialize(&dut)?.data() {
            Ok(v) => Ok(v),
            Err(_) => {
                if self.reg_id.is_some() {
                    match dut.get_register(self.reg_id.unwrap()) {
                        Ok(v) => Err(UndefinedDataError::py_err(format!("Attempted to reference data from register '{}' but it contains undefined (X) bits!", v.name))),
                        Err(_) => Err(UndefinedDataError::py_err("Attempted to reference a data value that contains undefined (X) bits!")),
                    }
                } else {
                    Err(UndefinedDataError::py_err(
                        "Attempted to reference a data value that contains undefined (X) bits!",
                    ))
                }
            }
        }
    }

    fn set_data(&self, value: BigUint) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.set_data(value);
        Ok(self.clone())
    }

    /// An alias for field()
    fn bit(&self, name: &str) -> PyResult<BitCollection> {
        self.field(name)
    }

    fn field(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();

        if self.whole_reg {
            let reg = dut.get_register(self.reg_id.unwrap())?;
            if reg.fields.contains_key(name) {
                Ok(BitCollection {
                    reg_id: self.reg_id,
                    field: Some(name.to_string()),
                    whole_reg: false,
                    whole_field: true,
                    bit_ids: reg.fields.get(name).unwrap().bit_ids(&dut),
                    i: 0,
                })
            } else {
                let msg = format!(
                    "Register '{}' does not have a bit field called '{}'",
                    reg.name, name
                );
                Err(PyErr::new::<exceptions::RuntimeError, _>(msg))
            }
        } else {
            Err(PyErr::new::<exceptions::RuntimeError, _>(
                "'.bits(<name>)' method can only be called on registers",
            ))
        }
    }

    /// Returns the fully resolved register address (with all parent base addresses applied)
    fn address(&self) -> PyResult<u128> {
        let dut = origen::dut();
        match self.reg(&dut) {
            Some(x) => Ok(x.address(&dut)),
            None => Err(PyErr::new::<exceptions::RuntimeError, _>(
                "Called 'address()' on a BitCollection that is not associated with a register",
            )),
        }
    }

    /// Returns the address offset (local) address
    fn offset(&self) -> PyResult<usize> {
        let dut = origen::dut();
        match self.reg(&dut) {
            Some(x) => Ok(x.offset),
            None => Err(PyErr::new::<exceptions::RuntimeError, _>(
                "Called 'offset()' on a BitCollection that is not associated with a register",
            )),
        }
    }
}

/// Internal helper methods
impl BitCollection {
    /// Turn into a full BitCollection containing bit object references
    fn materialize<'a>(&self, dut: &'a MutexGuard<Dut>) -> Result<RichBC<'a>> {
        if self.whole_reg {
            Ok(dut.get_register(self.reg_id.unwrap())?.bits(&dut))
        } else if self.whole_field {
            Ok(dut
                .get_register(self.reg_id.unwrap())?
                .fields
                .get(self.field.as_ref().unwrap())
                .unwrap()
                .bits(&dut))
        } else {
            Ok(RichBC::for_bit_ids(&self.bit_ids, &dut))
        }
    }

    /// Return the register object or None if the BitCollection does not have a reg_id attribute
    fn reg<'a>(&self, dut: &'a MutexGuard<Dut>) -> Option<&'a Register> {
        if self.reg_id.is_some() {
            Some(dut.get_register(self.reg_id.unwrap()).unwrap())
        } else {
            None
        }
    }
}
