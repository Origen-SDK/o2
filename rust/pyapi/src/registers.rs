mod address_block;
pub mod bit_collection;
mod memory_map;
mod register;
mod register_collection;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::sync::RwLock;

use origen::core::model::registers::bit::{UNDEFINED, ZERO};
use origen::core::model::registers::{Bit, BitOrder, SummaryField};
use origen::utility::big_uint_helpers::*;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use register::{Field, FieldEnum, ResetVal};

pub use register_collection::RegisterCollection;

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
    register_file_id: Option<usize>,
    name: &str,
    offset: usize,
    size: Option<usize>,
    bit_order: String,
    mut fields: Vec<PyRef<Field>>,
    filename: Option<String>,
    lineno: Option<usize>,
    description: Option<String>,
    resets: Option<Vec<PyRef<ResetVal>>>,
    access: Option<String>,
) -> PyResult<usize> {
    let reg_id;
    let reg_fields;
    let base_bit_id;
    let mut reset_vals: Vec<Option<(BigUint, Option<BigUint>)>> = Vec::new();
    let mut non_zero_reset = false;
    let lsb0;

    fields.sort_by_key(|field| field.offset);

    {
        base_bit_id = origen::dut().bits.len();
    }
    {
        let mut dut = origen::dut();
        reg_id = dut.create_reg(
            address_block_id,
            register_file_id,
            name,
            offset,
            size,
            &bit_order,
            filename,
            lineno,
            description,
        )?;
        let reg = dut.get_mut_register(reg_id)?;
        lsb0 = reg.bit_order == BitOrder::LSB0;
        for f in &fields {
            let acc = match &f.access {
                Some(x) => x,
                None => match &access {
                    Some(y) => y,
                    None => "rw",
                },
            };
            let field = reg.add_field(
                &f.name,
                f.description.as_ref(),
                f.offset,
                f.width,
                acc,
                f.filename.as_ref(),
                f.lineno,
            )?;
            for e in &f.enums {
                field.add_enum(&e.name, &e.description, &e.value)?;
            }
            if f.resets.is_none() && resets.is_none() {
                reset_vals.push(None);
            } else {
                let mut val_found = false;
                // Store any resets defined on the field
                if !f.resets.is_none() {
                    for r in f.resets.as_ref().unwrap() {
                        field.add_reset(&r.name, &r.value, r.mask.as_ref())?;
                        // Apply this state to the register at time 0
                        if r.name == "hard" {
                            //TODO: Need to handle the mask here too
                            reset_vals.push(Some((r.value.clone(), r.mask.clone())));
                            non_zero_reset = true;
                            val_found = true;
                        }
                    }
                }
                // Store the field's portion of any top-level register resets
                if !resets.is_none() {
                    for r in resets.as_ref().unwrap() {
                        // Allow a reset of the same name defined on a field to override
                        // it's portion of a top-level register reset value - i.e. if the field
                        // already has a value for this reset then do nothing here
                        if !field.resets.contains_key(&r.name) {
                            // Work out the portion of the reset for this field
                            let value =
                                bit_slice(&r.value, field.offset, field.width + field.offset - 1)?;
                            let mask = match &r.mask {
                                None => None,
                                Some(x) => Some(bit_slice(
                                    &x,
                                    field.offset,
                                    field.width + field.offset - 1,
                                )?),
                            };
                            field.add_reset(&r.name, &value, mask.as_ref())?;
                            // Apply this state to the register at time 0
                            if r.name == "hard" {
                                //TODO: Need to handle the mask here too
                                reset_vals.push(Some((value, mask)));
                                non_zero_reset = true;
                                val_found = true;
                            }
                        }
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
        reg_fields = reg.fields(true).collect::<Vec<SummaryField>>();
    }

    // Create the bits now that we know which ones are implemented
    if lsb0 {
        reset_vals.reverse();
    }
    let mut dut = origen::dut();
    for field in reg_fields {
        // Intention here is to skip decomposing the BigUint unless required
        if !non_zero_reset || field.spacer || reset_vals.last().unwrap().is_none() {
            for i in 0..field.width {
                let val;
                if field.spacer {
                    val = ZERO;
                } else {
                    val = UNDEFINED;
                }
                let id;
                {
                    id = dut.bits.len();
                }
                dut.bits.push(Bit {
                    id: id,
                    overlay: RwLock::new(None),
                    overlay_snapshots: RwLock::new(HashMap::new()),
                    register_id: reg_id,
                    state: RwLock::new(val),
                    device_state: RwLock::new(val),
                    state_snapshots: RwLock::new(HashMap::new()),
                    access: field.access,
                    position: field.offset + i,
                });
            }
        } else {
            let reset_val = reset_vals.last().unwrap().as_ref().unwrap();

            // If no reset mask to apply. There is a lot of duplication here but ran
            // into borrow issues that I couldn't resolve and had to move on.
            if reset_val.1.as_ref().is_none() {
                let mut bytes = reset_val.0.to_bytes_be();
                let mut byte = bytes.pop().unwrap();
                for i in 0..field.width {
                    let state = (byte >> i % 8) & 1;
                    let id;
                    {
                        id = dut.bits.len();
                    }
                    dut.bits.push(Bit {
                        id: id,
                        overlay: RwLock::new(None),
                        overlay_snapshots: RwLock::new(HashMap::new()),
                        register_id: reg_id,
                        state: RwLock::new(state),
                        device_state: RwLock::new(state),
                        state_snapshots: RwLock::new(HashMap::new()),
                        access: field.access,
                        position: field.offset + i,
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
                let mut mask_bytes = reset_val.1.as_ref().unwrap().to_bytes_be();
                let mut mask_byte = mask_bytes.pop().unwrap();
                for i in 0..field.width {
                    let state = (byte >> i % 8) & (mask_byte >> i % 8) & 1;
                    let id;
                    {
                        id = dut.bits.len();
                    }
                    dut.bits.push(Bit {
                        id: id,
                        overlay: RwLock::new(None),
                        overlay_snapshots: RwLock::new(HashMap::new()),
                        register_id: reg_id,
                        state: RwLock::new(state),
                        device_state: RwLock::new(state),
                        state_snapshots: RwLock::new(HashMap::new()),
                        access: field.access,
                        position: field.offset + i,
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
