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
    pub reg_id: usize,
    /// When true the BitCollection contains an entire register's worth of bits
    pub whole: bool,
    pub bit_ids: Vec<usize>,
    /// Iterator index
    pub i: usize,
}

/// Rust-private methods, i.e. not accessible from Python
impl BitCollection {
    pub fn from_reg_id(id: usize) -> BitCollection {
        BitCollection {
            reg_id: id,
            whole: true,
            bit_ids: Vec::new(),
            i: 0,
        }
    }
}

#[pyproto]
impl PyObjectProtocol for BitCollection {
    fn __repr__(&self) -> PyResult<String> {
        if self.whole {
            let dut = origen::dut();
            let reg = dut.get_register(self.reg_id)?;
            Ok(reg.console_display(&dut, None, true)?)
        } else {
            Ok(format!(
                "<BitCollection containing {} bits>",
                self.bit_ids.len()
            ))
        }
    }

    /// Implements my_reg.my_bits
    fn __getattr__(&self, query: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        if self.whole {
            let reg = dut.get_register(self.reg_id)?;
            if reg.fields.contains_key(query) {
                Ok(BitCollection {
                    reg_id: self.reg_id,
                    whole: false,
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

    /// An alias for get_data()
    fn data(&self) -> PyResult<BigUint> {
        self.get_data()
    }

    fn get_data(&self) -> PyResult<BigUint> {
        let dut = origen::dut();
        match self.materialize(&dut)?.data() {
            Ok(v) => Ok(v),
            Err(_) => {
                match dut.get_register(self.reg_id) {
                    Ok(v) => Err(UndefinedDataError::py_err(format!("Attempted to reference data from register '{}' but it contains undefined (X) bits!", v.name))),
                    Err(_) => Err(UndefinedDataError::py_err("Attempted to reference a data value that contains undefined (X) bits!")),
                }
            },
        }
    }

    fn set_data(&self, value: BigUint) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.set_data(value);
        Ok(self.clone())
    }

    fn bits(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        let reg = dut.get_register(self.reg_id)?;

        if self.whole {
            if reg.fields.contains_key(name) {
                Ok(BitCollection {
                    reg_id: self.reg_id,
                    whole: false,
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
        if self.whole {
            Ok(dut.get_register(self.reg_id)?.bits(&dut))
        } else {
            Ok(RichBC::for_bit_ids(&self.bit_ids, &dut))
        }
    }
}
