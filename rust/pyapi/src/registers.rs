mod address_block;
mod bit_collection;
mod memory_map;
mod register;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::sync::RwLock;

//use crate::dut::PyDUT;
//use origen::core::model::registers::Register;
use bit_collection::BitCollection;
use origen::core::model::registers::bit::{UNDEFINED, ZERO};
use origen::core::model::registers::{Bit, SummaryField};
use origen::DUT;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use register::{Field, FieldEnum, ResetVal};

#[pymodule]
/// Implements the module _origen.registers in Python
pub fn registers(_py: Python, m: &PyModule) -> PyResult<()> {
    // Used to pass register field info from Python to Rust when defining regs
    m.add_class::<Field>()?;
    m.add_class::<FieldEnum>()?;
    m.add_class::<ResetVal>()?;

    m.add_wrapped(wrap_pyfunction!(create))?;
    Ok(())
}

/// Create a new register, returning its ID
#[pyfunction]
fn create(
    address_block_id: usize,
    name: &str,
    offset: usize,
    size: Option<usize>,
    mut fields: Vec<&Field>,
) -> PyResult<usize> {
    let reg_id;
    let reg_fields;
    let base_bit_id;
    let mut reset_vals: Vec<Option<(&BigUint, Option<&BigUint>)>> = Vec::new();
    let mut non_zero_reset = false;

    fields.sort_by_key(|field| field.offset);

    {
        base_bit_id = origen::dut().bits.len();
    }
    {
        let mut dut = origen::dut();
        reg_id = dut.create_reg(address_block_id, name, offset, size)?;
        let reg = dut.get_mut_register(reg_id)?;
        for f in &fields {
            let field = reg.add_field(&f.name, &f.description, f.offset, f.width, &f.access)?;
            for e in &f.enums {
                field.add_enum(&e.name, &e.description, &e.value)?;
            }
            if f.resets.is_none() {
                reset_vals.push(None);
            } else {
                let mut val_found = false;
                for r in f.resets.as_ref().unwrap() {
                    field.add_reset(&r.name, &r.value, r.mask.as_ref())?;
                    // Apply this state to the register at time 0
                    if r.name == "hard" {
                        //TODO: Need to handle the mask here too
                        reset_vals.push(Some((&r.value, r.mask.as_ref())));
                        non_zero_reset = true;
                        val_found = true;
                    }
                }
                if !val_found {
                    reset_vals.push(None);
                }
            }
        }
        for i in 0..reg.size as usize {
            reg.bit_ids.push((base_bit_id + i) as usize);
        }
        reg_fields = reg.named_bits(true).collect::<Vec<SummaryField>>();
    }

    // Create the bits now that we know which ones are implemented
    reset_vals.reverse();
    let mut dut = origen::dut();
    for field in reg_fields {
        // Intention here is to skip decomposing the BigUint unless required
        if !non_zero_reset || field.spacer || reset_vals.last().unwrap().is_none() {
            for _i in 0..field.width {
                let val;
                if field.spacer {
                    val = ZERO;
                } else {
                    val = UNDEFINED;
                }
                dut.bits.push(Bit {
                    overlay: RwLock::new(None),
                    overlay_snapshots: RwLock::new(HashMap::new()),
                    register_id: reg_id,
                    state: RwLock::new(val),
                    reset_state: RwLock::new(val),
                    state_snapshots: RwLock::new(HashMap::new()),
                    access: field.access,
                });
            }
        } else {
            let reset_val = reset_vals.last().unwrap().unwrap();

            // If no reset mask to apply. There is a lot of duplication here but ran
            // into borrow issues that I couldn't resolve and had to move on.
            if reset_val.1.is_none() {
                let mut bytes = reset_val.0.to_bytes_be();
                let mut byte = bytes.pop().unwrap();
                for i in 0..field.width {
                    let state = (byte >> i % 8) & 1;
                    dut.bits.push(Bit {
                        overlay: RwLock::new(None),
                        overlay_snapshots: RwLock::new(HashMap::new()),
                        register_id: reg_id,
                        state: RwLock::new(state),
                        reset_state: RwLock::new(state),
                        state_snapshots: RwLock::new(HashMap::new()),
                        access: field.access,
                    });
                    if i % 8 == 7 {
                        match bytes.pop() {
                            Some(x) => byte = x,
                            None => byte = 0,
                        }
                    }
                }
            } else {
                let mut bytes = reset_val.0.to_bytes_be();
                let mut byte = bytes.pop().unwrap();
                let mut mask_bytes = reset_val.1.unwrap().to_bytes_be();
                let mut mask_byte = mask_bytes.pop().unwrap();
                for i in 0..field.width {
                    let state = (byte >> i % 8) & (mask_byte >> i % 8) & 1;
                    dut.bits.push(Bit {
                        overlay: RwLock::new(None),
                        overlay_snapshots: RwLock::new(HashMap::new()),
                        register_id: reg_id,
                        state: RwLock::new(state),
                        reset_state: RwLock::new(state),
                        state_snapshots: RwLock::new(HashMap::new()),
                        access: field.access,
                    });
                    if i % 8 == 7 {
                        match bytes.pop() {
                            Some(x) => byte = x,
                            None => byte = 0,
                        }
                        match mask_bytes.pop() {
                            Some(x) => mask_byte = x,
                            None => mask_byte = 0,
                        }
                    }
                }
            }
        }
        if !field.spacer {
            reset_vals.pop();
        }
    }

    Ok(reg_id)
}

///// Returns an empty register collection
//fn empty_regs() -> PyResult<Registers> {
//    Ok(Registers {
//        address_block_id: None,
//        register_file_id: None,
//        ids: None,
//        i: 0,
//    })
//}

///// Implements the user APIs my_block.[.my_memory_map][.my_address_block].reg() and
///// my_block.[.my_memory_map][.my_address_block].regs
//#[pymethods]
//impl PyDUT {
//    fn regs(&self, address_block_id: Option<usize>) -> PyResult<Registers> {
//        Ok(Registers {
//            address_block_id: address_block_id,
//            i: 0,
//        })
//    }
//
//    fn reg(&self, address_block_id: usize, name: &str) -> PyResult<BitCollection> {
//        Ok(BitCollection {
//            reg_id: DUT
//                .lock()
//                .unwrap()
//                .get_address_block(address_block_id)?
//                .get_register_id(name)?,
//            whole: true,
//            bit_numbers: Vec::new(),
//            i: 0,
//        })
//    }
//}

/// Implements the user API to work with a collection of registers. The collection could be associated
/// with another container object (an address block or register file), or this could be its own collection
/// of otherwise un-related registers.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Registers {
    /// The ID of the address block which contains these registers. It is optional so that
    /// an empty Registers collection can be created, or a collection of
    pub address_block_id: Option<usize>,
    /// The ID of the register file which contains these registers. It is optional as registers
    /// can be instantiated in an address block directly and are not necessarily within an
    pub register_file_id: Option<usize>,
    /// The IDs of the contained registers. If not present then the IDs will be derived from either
    /// the associated register file or address block. If both are defined then the register file IDs
    /// will be used.
    pub ids: Option<Vec<usize>>,
    /// Iterator index
    pub i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl Registers {
    fn len(&self) -> PyResult<usize> {
        if self.address_block_id.is_some() {
            Ok(DUT
                .lock()
                .unwrap()
                .get_address_block(self.address_block_id.unwrap())?
                .registers
                .len())
        } else {
            Ok(0)
        }
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let keys: Vec<String> = ab.registers.keys().map(|x| x.clone()).collect();
            Ok(keys)
        } else {
            Ok(Vec::new())
        }
    }

    fn values(&self) -> PyResult<Vec<BitCollection>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let values: Vec<BitCollection> = ab
                .registers
                .values()
                .map(|x| BitCollection::from_reg_id(*x, &dut))
                .collect();
            Ok(values)
        } else {
            Ok(Vec::new())
        }
    }

    fn items(&self) -> PyResult<Vec<(String, BitCollection)>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let items: Vec<(String, BitCollection)> = ab
                .registers
                .iter()
                .map(|(k, v)| (k.to_string(), BitCollection::from_reg_id(*v, &dut)))
                .collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }
}
