use num_bigint::BigUint;
use origen::core::model::registers::BitCollection as RichBC;
use origen::Dut;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions;
use pyo3::exceptions::AttributeError;
use pyo3::import_exception;
use pyo3::prelude::*;
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
    pub fn from_reg_id(id: usize) -> BitCollection {
        BitCollection {
            reg_id: Some(id),
            field: None,
            whole_reg: true,
            whole_field: false,
            bit_ids: Vec::new(),
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
                        "<BitCollection containing an ad-hoc colleciton of {} bit(s) from register '{}'>",
                        self.bit_ids.len(),
                        reg.name
                    ))
                }
            } else {
                if self.whole_field {
                    Ok(format!(
                        "<BitCollection containing all bits from register field '{}.{}'>",
                        reg.name,
                        self.field.as_ref().unwrap()
                    ))
                } else {
                    Ok(format!(
                        "<BitCollection containing an ad-hoc collection of {} bit(s) from register field '{}.{}'>",
                        self.bit_ids.len(),
                        reg.name,
                        self.field.as_ref().unwrap()
                    ))
                }
            }
        } else {
            Ok(format!(
                "<BitCollection containing an ad-hoc collection of {} bit(s)>",
                self.bit_ids.len()
            ))
        }
    }

    /// Implements my_reg.my_bits
    fn __getattr__(&self, query: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        if self.whole_reg {
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
        } else {
            Err(AttributeError::py_err(format!(
                "'BitCollection' object has no attribute '{}'",
                query
            )))
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
        Ok(self.clone())
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

    fn bits(&self, name: &str) -> PyResult<BitCollection> {
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
                ".bits(<name>) method can only be called on registers",
            ))
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
}
