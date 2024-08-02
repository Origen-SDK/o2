use itertools::Itertools;
use num_bigint::BigUint;
use origen::core::model::registers::bit::Overlay as OrigenBitOverlay;
use origen::core::model::registers::BitCollection as RichBC;
use origen::core::model::registers::{BitOrder, Field, Register};
use origen::{Dut, Result, TEST};
use pyo3::exceptions;
use pyo3::exceptions::PyAttributeError;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyInt, PyList, PySlice, PyString, PyTuple};
use std::iter::FromIterator;
use std::sync::MutexGuard;

import_exception!(origen.errors, UndefinedDataError);

#[pyclass]
pub struct Overlay {
    label: Option<String>,
    symbol: Option<String>,
    persistent: bool,
}

#[pymethods]
impl Overlay {
    #[getter]
    fn label(&self) -> Option<String> {
        self.label.clone()
    }

    #[getter]
    fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }

    #[getter]
    fn persistent(&self) -> bool {
        self.persistent
    }
}

impl Overlay {
    fn from_origen(overlay: &OrigenBitOverlay) -> Self {
        Self {
            label: overlay.label.clone(),
            symbol: overlay.symbol.clone(),
            persistent: overlay.persistent,
        }
    }
}

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
    /// When true the BitCollection is representing a gap in a register
    pub spacer: bool,
    /// This is a temporary bit_order that controls what view the user sees via
    /// the with_msb0() and with_lsb0() methods
    pub bit_order: BitOrder,
    /// Iterator index and vars
    pub i: usize,
    pub shift_left: bool,
    pub shift_logical: bool,
    pub transaction: u8,
}

/// User API methods
#[pymethods]
impl BitCollection {
    /// Similar to the regular clone(), but doesn't clone the bit_ids vector and assigns
    /// the supplied one instead.
    pub fn smart_clone(&self, bit_ids: Vec<usize>) -> BitCollection {
        BitCollection {
            reg_id: self.reg_id,
            field: match &self.field {
                None => None,
                Some(x) => Some(x.clone()),
            },
            whole_reg: self.whole_reg && self.bit_ids.len() == bit_ids.len(),
            whole_field: self.whole_field && self.bit_ids.len() == bit_ids.len(),
            bit_ids: bit_ids,
            i: 0,
            shift_left: self.shift_left,
            shift_logical: self.shift_logical,
            spacer: self.spacer,
            bit_order: self.bit_order,
            transaction: self.transaction,
        }
    }

    /// Returns a bit collection containing the given bit indices
    #[pyo3(signature=(*args))]
    fn subset(&self, args: &PyTuple) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        let mut bit_ids: Vec<usize> = Vec::new();

        for arg in args.iter() {
            let id = arg.extract::<usize>();
            if id.is_ok() {
                bit_ids.push(id.unwrap());
            } else {
                let ids = arg.extract::<Vec<usize>>();
                if ids.is_ok() {
                    for id in ids.unwrap() {
                        bit_ids.push(id);
                    }
                } else {
                    let msg = format!(
                        "The BitCollection.subset() method can accept multiple integers or an array of integer arguments only, this invalid: '{:?}'",
                        args
                    );
                    return Err(PyErr::new::<exceptions::PyRuntimeError, _>(msg));
                }
            }
        }

        bc.bit_ids = bit_ids
            .into_iter()
            .sorted()
            .map(|x| self.bit_ids[x])
            .collect::<Vec<usize>>();

        bc.whole_reg = false;
        bc.whole_field = false;
        Ok(bc)
    }

    /// Returns a copy of the BitCollection with its bit_order attribute set to MSB0
    fn with_msb0(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        bc.bit_order = BitOrder::MSB0;
        Ok(bc)
    }

    /// Returns a copy of the BitCollection with its bit_order attribute set to LSB0
    fn with_lsb0(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        bc.bit_order = BitOrder::LSB0;
        Ok(bc)
    }

    fn len(&self) -> PyResult<usize> {
        Ok(self.bit_ids.len())
    }

    #[getter]
    fn size(&self) -> PyResult<usize> {
        self.len()
    }

    #[getter]
    fn width(&self) -> PyResult<usize> {
        self.len()
    }

    #[getter]
    /// Returns the bit's position (offset) within its parent register.
    /// If the BitCollection contains > 1 bits, then this will return the lowest position.
    fn position(&self) -> PyResult<usize> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.position())
    }

    #[getter]
    /// Returns the access attribute of the BitCollection. This will raise an error if
    /// the collection is comprised of bits with a different access attribute value.
    fn access(&self) -> PyResult<String> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.access()?.to_string())
    }

    fn shift_out_left(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        bc.i = 0;
        bc.shift_left = true;
        bc.shift_logical = false;
        Ok(bc)
    }

    fn shift_out_right(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        bc.i = 0;
        bc.shift_left = false;
        bc.shift_logical = false;
        Ok(bc)
    }

    fn reset(&self, name: Option<&str>) -> PyResult<BitCollection> {
        let dut = origen::dut();
        match name {
            Some(n) => self.materialize(&dut)?.reset(n, &dut)?,
            None => self.materialize(&dut)?.reset("hard", &dut)?,
        };
        Ok(self.clone())
    }

    fn reset_val(&self, name: Option<&str>) -> PyResult<Option<BigUint>> {
        let dut = origen::dut();
        match name {
            Some(n) => Ok(self.materialize(&dut)?.reset_val(n, &dut)?),
            None => Ok(self.materialize(&dut)?.reset_val("hard", &dut)?),
        }
    }

    /// Returns true if the data value of any of the bits has been changed since
    /// the last reset. It returns true even if the current data value matches the
    /// default reset value and it will only be returned to false upon a reset operation.
    fn is_modified_since_reset(&self) -> PyResult<bool> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.is_modified_since_reset())
    }

    /// Returns true if the data value of all bits matches that of the given
    /// reset type ("hard", by default).
    fn is_in_reset_state(&self, name: Option<&str>) -> PyResult<bool> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.is_in_reset_state(name, &dut)?)
    }

    /// Take a snapshot of the current state of all bits, the state can be rolled
    /// back in future by supplying the same name to the rollback method
    fn snapshot(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.snapshot(name)?;
        Ok(self.clone())
    }

    /// Returns true if the state of any bits has changed vs. the given snapshot
    /// reference. An error will be raised if no snapshot with the given name is found.
    fn is_changed(&self, name: &str) -> PyResult<bool> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.is_changed(name)?)
    }

    /// Rollback the state of all bits to the given snapshot.
    /// An error will be raised if no snapshot with the given name is found.
    fn rollback(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.rollback(name)?;
        Ok(self.clone())
    }

    #[pyo3(signature=(shift_in=0))]
    fn shift_left(&self, shift_in: u8) -> PyResult<u8> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.shift_left(shift_in)?)
    }

    #[pyo3(signature=(shift_in=0))]
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
                        Ok(v) => Err(UndefinedDataError::new_err(format!("Attempted to reference data from register '{}' but it contains undefined (X) bits!", v.name))),
                        Err(_) => Err(UndefinedDataError::new_err("Attempted to reference a data value that contains undefined (X) bits!")),
                    }
                } else {
                    Err(UndefinedDataError::new_err(
                        "Attempted to reference a data value that contains undefined (X) bits!",
                    ))
                }
            }
        }
    }

    fn set_data(&self, value: BigUint) -> PyResult<BitCollection> {
        let dut = origen::dut();
        let rbc = self.materialize(&dut)?;
        rbc.set_data(value);
        if self.is_verify_transaction_open() {
            rbc.set_verify_flag(None)?;
        }
        Ok(self.clone())
    }

    #[pyo3(signature=(label=None, symbol=None, mask=None))]
    fn set_overlay(
        &self,
        label: Option<String>,
        symbol: Option<String>,
        mask: Option<BigUint>,
    ) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?
            .set_overlay(label, symbol, mask, true);
        Ok(self.clone())
    }

    fn clear_overlay(&self) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.clear_persistent_overlay();
        Ok(self.clone())
    }

    fn set_capture(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.capture();
        Ok(self.clone())
    }

    fn clear_capture(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.clear_capture();
        Ok(self.clone())
    }

    fn get_overlay(&self) -> PyResult<Option<Overlay>> {
        let dut = origen::dut();
        Ok(match self.materialize(&dut)?.get_overlay()? {
            Some(o) => Some(Overlay::from_origen(&o)),
            None => None,
        })
    }

    fn copy(&self, src: &BitCollection) -> PyResult<BitCollection> {
        let dut = origen::dut();
        let dest = self.materialize(&dut)?;
        let source = src.materialize(&dut)?;

        for (i, bit) in dest.bits.iter().enumerate() {
            if i < source.bits.len() {
                bit.copy_state(source.bits[i]);
            }
        }
        Ok(self.clone())
    }

    /// Returns a representation of the register which owns the bits in the collection
    fn as_reg(&self) -> PyResult<BitCollection> {
        let dut = origen::dut();
        if let Some(id) = self.reg_id {
            Ok(BitCollection::from_reg_id(id, &dut))
        } else {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Called as_reg() on a bit collection with no association to a register",
            ))
        }
    }

    #[pyo3(signature=(enable=None, preset=false))]
    /// Trigger a verify transaction on the register
    pub fn _internal_verify(
        &self,
        enable: Option<BigUint>,
        preset: bool,
    ) -> PyResult<Option<usize>> {
        if self.transaction != 0 {
            self.set_verify_flag(enable)?;
            Ok(None)
        } else {
            let dut = origen::dut();
            let ref_id = self.materialize(&dut)?.verify(enable, preset, &dut)?;
            Ok(ref_id)
        }
    }

    pub fn _end_internal_verify(&self, ref_id: usize) -> PyResult<()> {
        TEST.close(ref_id)?;
        Ok(())
    }

    #[pyo3(signature=(enable=None))]
    /// Equivalent to calling verify() but without invoking a register transaction at the end,
    /// i.e. it will set the verify flag on the bits and optionally apply an enable mask when
    /// deciding what bit flags to set.
    pub fn set_verify_flag(&self, enable: Option<BigUint>) -> PyResult<BitCollection> {
        let dut = origen::dut();
        self.materialize(&dut)?.set_verify_flag(enable)?;
        Ok(self.clone())
    }

    pub fn _internal_write(&self) -> PyResult<Option<usize>> {
        if self.transaction != 0 {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Can't call write() from within a transaction block, did you mean to call set_data()?",
            ))
        } else {
            let dut = origen::dut();
            let ref_id = self.materialize(&dut)?.write(&dut)?;
            Ok(ref_id)
        }
    }

    pub fn _end_internal_write(&self, ref_id: usize) -> PyResult<()> {
        TEST.close(ref_id)?;
        Ok(())
    }

    pub fn is_verify_transaction_open(&self) -> bool {
        self.transaction == 1
    }

    pub fn is_write_transaction_open(&self) -> bool {
        self.transaction == 2
    }

    pub fn is_transaction_open(&self) -> bool {
        self.transaction != 0
    }

    pub fn _internal_start_verify_transaction(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        if bc.transaction != 0 {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Attempted to start a verify transaction on a BitCollection that already has a transaction underway",
            ))
        } else {
            bc.transaction = 1;
            Ok(bc)
        }
    }

    pub fn _internal_end_verify_transaction(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        if self.transaction != 1 {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Attempted to end a verify transaction on a BitCollection that does not have a transaction underway",
            ))
        } else {
            bc.transaction = 0;
            Ok(bc)
        }
    }

    pub fn _internal_start_write_transaction(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        if bc.transaction != 0 {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Attempted to start a write transaction on a BitCollection that already has a transaction underway",
            ))
        } else {
            bc.transaction = 2;
            Ok(bc)
        }
    }

    pub fn _internal_end_write_transaction(&self) -> PyResult<BitCollection> {
        let mut bc = self.clone();
        if self.transaction != 2 {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Attempted to end a write transaction on a BitCollection that does not have a transaction underway",
            ))
        } else {
            bc.transaction = 0;
            Ok(bc)
        }
    }

    /// Clears the verify flag on all bits in the collection
    pub fn clear_verify_flag(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.clear_verify_flag();
        Ok(self.clone())
    }

    /// Returns true if any bits in the collection has their verify flag set
    pub fn is_to_be_verified(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_to_be_verified())
    }

    /// Returns true if any bits in the collection has their capture flag set
    pub fn is_to_be_captured(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_to_be_captured())
    }

    /// Returns true if any bits in the collection has an overlay set
    pub fn has_overlay(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.has_overlay())
    }

    /// Returns true if any bits in the collection are writeable
    pub fn is_writeable(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_writeable())
    }

    pub fn is_writable(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_writable())
    }

    /// Returns true if any bits in the collection are readable
    pub fn is_readable(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_readable())
    }

    /// Returns true if the current data state of te BitCollection is out of sync with
    /// what the device state is
    pub fn is_update_required(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.is_update_required())
    }

    /// Set the collection's device_state field to be the same as its current data state
    pub fn update_device_state(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.update_device_state()?;
        Ok(self.clone())
    }

    fn field(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();

        if self.whole_reg {
            let reg = dut.get_register(self.reg_id.unwrap())?;
            if reg.fields.contains_key(name) {
                let mut bc = self.smart_clone(reg.fields.get(name).unwrap().bit_ids(&dut));
                bc.field = Some(name.to_string());
                bc.whole_field = true;
                Ok(bc)
            } else {
                let msg = format!(
                    "Register '{}' does not have a bit field called '{}'",
                    reg.name, name
                );
                Err(PyErr::new::<exceptions::PyRuntimeError, _>(msg))
            }
        } else {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "'.field()' method can only be called on registers",
            ))
        }
    }

    #[pyo3(signature=(*args))]
    fn try_fields(&self, args: &PyTuple) -> PyResult<BitCollection> {
        let dut = origen::dut();

        if self.whole_reg {
            let mut found = 0;
            let reg = dut.get_register(self.reg_id.unwrap())?;

            let names = Vec::from_iter(
                args.iter()
                    .map(|x| x.extract::<&str>().unwrap())
                    .collect::<Vec<&str>>(),
            );
            let mut bc = RichBC::default();
            bc.reg_id = Some(reg.id);

            for name in names {
                let field = reg.fields.get(name);
                if field.is_some() {
                    found += 1;
                    bc.field = Some(name.to_string());
                    let field = field.unwrap();
                    for bit in field.bits(&dut) {
                        bc.bits.push(bit)
                    }
                }
            }

            bc.sort_bits();

            if found == 0 {
                let msg = format!(
                    "Register '{}' does not have any bit fields called {:?}",
                    reg.name, args
                );
                Err(PyErr::new::<exceptions::PyRuntimeError, _>(msg))
            } else {
                if found > 1 {
                    bc.field = None;
                }
                Ok(BitCollection::from_rich_bc(&bc))
            }
        } else {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "'.try_fields()' method can only be called on registers",
            ))
        }
    }

    /// Returns the fully resolved register address (with all parent base addresses applied)
    fn address(&self) -> PyResult<u128> {
        let dut = origen::dut();
        match self.reg(&dut) {
            Some(x) => Ok(x.address(&dut, None)?),
            None => Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Called 'address()' on a BitCollection that is not associated with a register",
            )),
        }
    }

    /// Returns the address offset (local) address
    fn offset(&self) -> PyResult<usize> {
        let dut = origen::dut();
        match self.reg(&dut) {
            Some(x) => Ok(x.offset),
            None => Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Called 'offset()' on a BitCollection that is not associated with a register",
            )),
        }
    }

    fn status_str(&self, operation: &str) -> PyResult<String> {
        Ok(self.materialize(&origen::dut())?.status_str(operation)?)
    }

    fn clear_flags(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.clear_flags();
        Ok(self.clone())
    }

    fn set_undefined(&self) -> PyResult<BitCollection> {
        self.materialize(&origen::dut())?.set_undefined();
        Ok(self.clone())
    }

    fn verify_enables(&self) -> PyResult<BigUint> {
        Ok(self.materialize(&origen::dut())?.verify_enables())
    }

    fn capture_enables(&self) -> PyResult<BigUint> {
        Ok(self.materialize(&origen::dut())?.capture_enables())
    }

    fn overlay_enables(&self) -> PyResult<BigUint> {
        Ok(self.materialize(&origen::dut())?.overlay_enables())
    }

    /// Returns true if no contained bits are in X or Z state
    fn has_known_value(&self) -> PyResult<bool> {
        Ok(self.materialize(&origen::dut())?.has_known_value())
    }

    #[getter]
    /// Returns the name of the file (full path) where the register was defined.
    /// If it returns None it means that the register has been defined in a non-std location and Origen
    /// couldn't work out where the sourcefile is.
    fn filename(&self) -> PyResult<Option<String>> {
        match self.reg(&origen::dut()) {
            Some(x) => match &x.filename {
                Some(y) => Ok(Some(y.to_string())),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn description(&self) -> PyResult<Option<String>> {
        let mut dut = origen::dut();
        if self.whole_reg {
            let filename;
            let lineno;
            {
                let reg = self.reg(&dut).unwrap();
                if reg.description.is_some() {
                    return Ok(Some(reg.description.as_ref().unwrap().to_string()));
                } else if reg.filename.is_none() {
                    return Ok(None);
                }
                filename = reg.filename.as_ref().unwrap().clone();
                lineno = reg.lineno.as_ref().unwrap().clone();
            };
            Ok(dut.get_reg_description(&filename, lineno))
        } else if self.whole_field {
            let filename;
            let lineno;
            {
                let field = self.field_obj(&dut).unwrap();
                if field.description.is_some() {
                    return Ok(Some(field.description.as_ref().unwrap().to_string()));
                } else if field.filename.is_none() {
                    return Ok(None);
                }
                filename = field.filename.as_ref().unwrap().clone();
                lineno = field.lineno.as_ref().unwrap().clone();
            };
            Ok(dut.get_reg_description(&filename, lineno))
        } else {
            Ok(None)
        }
    }

    fn model_path(&self) -> PyResult<String> {
        let dut = origen::dut();
        match self.reg(&dut) {
            Some(x) => Ok(x.model_path(&dut)?),
            None => Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Called 'model_path()' on a BitCollection that is not associated with a register",
            )),
        }
    }

    /// Locates the "closest" controller to this bitcollection.
    /// "Closest" being defined as the first subblock (or the DUT) which implements a "verify/write_register"
    /// that owns the memory map -> address block -> register -> bit collection <- self
    fn controller_for(slf: &PyCell<Self>, py: Python, operation: Option<&str>) -> PyResult<PyObject> {
        let bc = slf.extract::<PyRef<Self>>()?;
        match bc._controller_for(py, operation)? {
            Some(c) => Ok(c),
            None => {
                Ok(py.None())
            }
        }
    }

    #[pyo3(signature=(data=None, **_kwargs))]
    fn write(
        slf: &PyCell<Self>,
        py: Python,
        data: Option<BigUint>,
        _kwargs: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        // Extract the underlying Rust object
        let bc = slf.extract::<PyRef<Self>>()?;

        // Update the data in the bitcollection
        if let Some(d) = data {
            bc.set_data(d)?;
        }

        // Attempt to find a controller which implements "write_register"
        match bc._controller_for(py, Some("write_register"))? {
            Some(c) => {
                // If we've found a matching controller, write the register
                let args = PyTuple::new(py, &[slf.to_object(py)]);
                c.call_method(py, "write_register", args, None)?;
            },
            None => return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                "No controller in the path {} implements a 'write_register'. Cannot write this register.",
                bc.model_path()?
            ))),
        }
        Ok(slf.into())
    }

    #[pyo3(signature=(data=None, **_kwargs))]
    fn verify(
        slf: &PyCell<Self>,
        py: Python,
        data: Option<BigUint>,
        _kwargs: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        // Extract the underlying Rust object
        let bc = slf.extract::<PyRef<Self>>()?;

        // Update the data in the bitcollection
        if let Some(d) = data {
            bc.set_data(d)?;
        }
        bc.set_verify_flag(None)?;

        // Attempt to find a controller which implements "verify_register"
        match bc._controller_for(py, Some("verify_register"))? {
            Some(c) => {
                // If we've found a matching controller, verify the register
                let args = PyTuple::new(py, &[slf.to_object(py)]);
                c.call_method(py, "verify_register", args, None)?;
            },
            None => return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                "No controller in the path {} implements a 'verify_register'. Cannot verify this register.",
                bc.model_path()?
            ))),
        }
        Ok(slf.into())
    }

    #[pyo3(signature=(**kwargs))]
    fn capture(slf: &PyCell<Self>, py: Python, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        // let bc = slf.extract::<PyRef<Self>>()?;
        // bc.capture();
        //slf.materialize(&origen::dut())?.capture();
        {
            let slf_bc = slf.extract::<PyRefMut<Self>>()?;
            slf_bc.materialize(&origen::dut())?.capture();
        }
        let bc = slf.extract::<PyRef<Self>>()?;

        // Attempt to find a controller which implements "capture_register"
        match bc._controller_for(py, Some("capture_register"))? {
            Some(c) => {
                // If we've found a matching controller, capture the register
                let args = PyTuple::new(py, &[slf.to_object(py)]);
                c.call_method(py, "capture_register", args, kwargs)?;
            }
            None => {
                // No controller specifies a "capture_register" method, so fall back to
                // using verify with the capture bits set and no additional arguments which
                // may change its state.
                match bc._controller_for(py, Some("verify_register"))? {
                    Some(c) => {
                        let args = PyTuple::new(py, &[slf.to_object(py)]);
                        c.call_method(py, "verify_register", args, None)?;
                    }
                    None => {
                        return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                            "No controller in the path {} implements a 'capture_register' or a 'verify_register'. Cannot capture this register.",
                            bc.model_path()?
                        )));
                    }
                }
            }
        }

        // Ok(self.clone())
        Ok(slf.into())
    }

    /// Just returns self un-modified. The configuration of the iteration
    /// index and any other iterator vars should be done by calling one of the methods
    /// like shift_out_left() and then iterating on the BitCollection object returned
    /// by that.
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<BitCollection> {
        Ok(slf.clone())
    }

    /// The BitCollection iterators always yield more BitCollections, usually a BC
    /// containing only one bit.
    // It's easier to implement a single API that way rather than adding another one
    // for Bit objects that is almost the same as the BC API.
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<BitCollection>> {
        if slf.i >= slf.bit_ids.len() {
            return Ok(None);
        }

        let mut bit_ids: Vec<usize> = Vec::new();
        if slf.shift_left {
            bit_ids.push(slf.bit_ids[slf.bit_ids.len() - slf.i - 1]);
        } else {
            bit_ids.push(slf.bit_ids[slf.i]);
        }

        let bc = BitCollection {
            reg_id: slf.reg_id,
            field: match &slf.field {
                Some(x) => Some(x.to_string()),
                None => None,
            },
            whole_reg: slf.whole_reg && slf.bit_ids.len() == bit_ids.len(),
            whole_field: slf.whole_field && slf.bit_ids.len() == bit_ids.len(),
            bit_ids: bit_ids,
            i: 0,
            shift_left: false,
            shift_logical: false,
            spacer: false,
            bit_order: slf.bit_order,
            transaction: slf.transaction,
        };

        slf.i += 1;
        Ok(Some(bc))
    }

    fn __repr__(&self) -> PyResult<String> {
        let plural;
        if self.bit_ids.len() > 1 {
            plural = "s";
        } else {
            plural = "";
        }
        if self.reg_id.is_some() {
            let dut = origen::dut();
            let reg = dut.get_register(self.reg_id.unwrap())?;
            if self.field.is_none() {
                if self.whole_reg {
                    Ok(reg.console_display(&dut, Some(self.bit_order), true)?)
                } else {
                    Ok(format!(
                        "<BitCollection: {} bit{} from '{}'>",
                        self.bit_ids.len(),
                        plural,
                        reg.name,
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
                        "<BitCollection: {} bit{} from '{}.{}'>",
                        self.bit_ids.len(),
                        plural,
                        reg.name,
                        self.field.as_ref().unwrap(),
                    ))
                }
            }
        } else {
            Ok(format!(
                "<BitCollection: {} bit{}>",
                self.bit_ids.len(),
                plural
            ))
        }
    }

    fn __getattr__(&self, py: Python, query: &str) -> PyResult<PyObject> {
        let dut = origen::dut();
        // .bits returns a Python list containing individual bit objects wrapped in BCs
        if query == "bits" {
            let mut bits: Vec<PyObject> = Vec::new();
            for id in &self.bit_ids {
                let bc = self.smart_clone(vec![*id]);
                bits.push(Py::new(py, bc)?.to_object(py));
            }
            Ok(PyList::new(py, bits).into())
        // .fields returns a Python dict containing field objects as BCs and spacer BCs
        } else if self.whole_reg {
            if query == "fields" {
                let fields = PyDict::new(py);
                let reg = dut.get_register(self.reg_id.unwrap())?;
                for field in reg.fields(true) {
                    let mut bc = self.smart_clone(Vec::from_iter(
                        self.bit_ids[field.offset..field.offset + field.width]
                            .iter()
                            .cloned(),
                    ));
                    bc.field = Some(field.name.clone());
                    bc.whole_field = true;
                    fields.set_item(field.name, Py::new(py, bc)?.to_object(py))?;
                }
                Ok(fields.into())
            // .my_field
            } else {
                let reg = dut.get_register(self.reg_id.unwrap())?;
                if reg.fields.contains_key(query) {
                    let mut bc = self.smart_clone(reg.fields.get(query).unwrap().bit_ids(&dut));
                    bc.field = Some(query.to_string());
                    bc.whole_field = true;
                    Ok(Py::new(py, bc)?.to_object(py))
                } else {
                    Err(PyAttributeError::new_err(format!(
                        "'BitCollection' object has no attribute '{}'",
                        query
                    )))
                }
            }
        } else {
            Err(PyAttributeError::new_err(format!(
                "'BitCollection' object has no attribute '{}'",
                query
            )))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.bit_ids.len())
    }

    /// Implements my_reg[5]
    fn __getitem__(&self, idx: &PyAny) -> PyResult<BitCollection> {
        let field = match &self.field {
            Some(x) => Some(x.to_string()),
            None => None,
        };
        if let Ok(slice) = idx.downcast::<PySlice>() {
            // Indices requires (what I think is) a max size. Should be plenty.
            let indices = slice.indices(8192)?;
            // TODO: Should this support step size?
            let mut upper;
            let mut lower;
            if indices.start > indices.stop {
                upper = indices.start as usize;
                lower = indices.stop as usize;
            } else {
                upper = indices.stop as usize;
                lower = indices.start as usize;
            }
            let bit_ids;
            if self.bit_order == BitOrder::MSB0 {
                upper = self.bit_ids.len() - upper - 1;
                lower = self.bit_ids.len() - lower - 1;
                let tmp = upper;
                upper = lower;
                lower = tmp;
            }
            bit_ids = Vec::from_iter(self.bit_ids[lower..upper + 1].iter().cloned());
            let mut bc = self.smart_clone(bit_ids);
            bc.field = field;
            Ok(bc)
        } else if let Ok(_int) = idx.downcast::<PyInt>() {
            let i = idx.extract::<usize>().unwrap();
            if i < self.bit_ids.len() {
                let mut bit_ids: Vec<usize> = Vec::new();
                if self.bit_order == BitOrder::LSB0 {
                    bit_ids.push(self.bit_ids[i]);
                } else {
                    bit_ids.push(self.bit_ids[self.bit_ids.len() - i - 1]);
                }
                let mut bc = self.smart_clone(bit_ids);
                bc.field = field;
                Ok(bc)
            } else {
                Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                    "The given bit index is out of range",
                ))
            }
        } else if let Ok(_name) = idx.downcast::<PyString>() {
            if self.whole_reg {
                let name = idx.extract::<&str>().unwrap();
                self.field(name)
            } else {
                Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                    "Illegal bit index given",
                ))
            }
        } else {
            Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Illegal bit index given",
            ))
        }
    }
}

/// Internal helper methods
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
            shift_left: false,
            shift_logical: false,
            spacer: false,
            // Important, all displays are LSB0 by default regardless of how the reg
            // was defined
            bit_order: BitOrder::LSB0,
            transaction: 0,
        }
    }

    pub fn from_rich_bc(bc: &RichBC) -> BitCollection {
        BitCollection {
            reg_id: bc.reg_id,
            field: match &bc.field {
                None => None,
                Some(x) => Some(x.clone()),
            },
            whole_reg: bc.whole_reg,
            whole_field: bc.whole_field,
            bit_ids: bc.bits.iter().map(|bit| bit.id).collect(),
            i: 0,
            shift_left: false,
            shift_logical: false,
            spacer: false,
            bit_order: BitOrder::LSB0,
            transaction: 0,
        }
    }

    /// Turn into a full BitCollection containing bit object references
    pub fn materialize<'a>(&self, dut: &'a MutexGuard<Dut>) -> Result<RichBC<'a>> {
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

    /// Return the field object or None
    fn field_obj<'a>(&self, dut: &'a MutexGuard<Dut>) -> Option<&'a Field> {
        if self.reg_id.is_some() && self.field.is_some() {
            let reg = dut.get_register(self.reg_id.unwrap()).unwrap();
            Some(reg.fields.get(self.field.as_ref().unwrap()).unwrap())
        } else {
            None
        }
    }

    fn _controller_for(&self, py: Python, operation: Option<&str>) -> PyResult<Option<PyObject>> {
        let mut ops: Vec<String> = vec![];
        if let Some(s) = operation {
            ops.push(s.to_string());
        } else {
            ops.push("write_register".to_string());
            ops.push("read_register".to_string());
        }
        let mut mp = self.model_path()?;
        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?.to_object(py))?;
        locals.set_item("dut", py.eval("origen.dut", Some(locals), None)?)?;
        let mut dut_checked = false;
        while !dut_checked {
            // Grab the subblock from the Python heap and see if it implements read/write register
            let m = py.eval(&mp, Some(locals), None)?;
            if mp == "dut" {
                dut_checked = true;
            }
            if ops.iter().all(|op| m.hasattr(op.as_str()).unwrap()) {
                // Found the controller. Return this.
                return Ok(Some(m.to_object(py)));
            } else {
                // Not this controller. Bump up to its parent and try again.
                mp = mp.rsplitn(2, ".").last().unwrap().to_string();
            }
        }
        // No controller found. Return None
        Ok(None)
    }
}
