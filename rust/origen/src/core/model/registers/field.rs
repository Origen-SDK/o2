use super::bit::UNDEFINED;
use super::{AccessType, BitCollection};
use crate::Dut;
use crate::Result;
use indexmap::map::IndexMap;
use num_bigint::BigUint;
use std::sync::MutexGuard;

#[derive(Debug)]
/// Named collections of bits within a register
pub struct Field {
    pub reg_id: usize,
    pub name: String,
    pub description: Option<String>,
    /// Offset from the start of the register in bits.
    pub offset: usize,
    /// Width of the field in bits.
    pub width: usize,
    pub access: AccessType,
    /// Contains any reset values defined for this field, if
    /// not present it will default to resetting all bits to undefined
    pub resets: IndexMap<String, Reset>,
    pub enums: IndexMap<String, EnumeratedValue>,
    pub related_fields: usize,
    /// The (Python) source file where the field was defined
    pub filename: Option<String>,
    /// The (Python) source file line number where the field was defined
    pub lineno: Option<usize>,
}

impl Field {
    pub fn add_enum(
        &mut self,
        name: &str,
        description: &str,
        value: &BigUint,
    ) -> Result<&EnumeratedValue> {
        //let acc: AccessType = match access.parse() {
        //    Ok(x) => x,
        //    Err(msg) => return Err(Error::new(&msg)),
        //};
        let e = EnumeratedValue {
            name: name.to_string(),
            description: description.to_string(),
            value: value.clone(),
        };
        self.enums.insert(name.to_string(), e);
        Ok(&self.enums[name])
    }

    pub fn add_reset(
        &mut self,
        name: &str,
        value: &BigUint,
        mask: Option<&BigUint>,
    ) -> Result<&Reset> {
        let r;
        if mask.is_some() {
            r = Reset {
                value: value.clone(),
                mask: Some(mask.unwrap().clone()),
            };
        } else {
            r = Reset {
                value: value.clone(),
                mask: None,
            };
        }
        self.resets.insert(name.to_string(), r);
        Ok(&self.resets[name])
    }

    /// Returns the bit IDs associated with the field, wrapped in a Vec
    pub fn bit_ids(&self, dut: &MutexGuard<Dut>) -> Vec<usize> {
        let mut bits: Vec<usize> = Vec::new();
        let reg = dut.get_register(self.reg_id).unwrap();

        if self.related_fields > 0 {
            // Collect all related fields
            let mut fields: Vec<&Field> = Vec::new();

            fields.push(self);

            for i in 0..self.related_fields {
                let f = reg.fields.get(&format!("{}{}", self.name, i + 1)).unwrap();
                fields.push(f);
            }

            // Sort them by offset
            //fields.sort_by(|a, b| b.offset.cmp(&a.offset));
            fields.sort_by_key(|f| f.offset);

            // Now collect their bits

            for f in fields {
                for i in 0..f.width {
                    bits.push(reg.bit_ids[f.offset + i]);
                }
            }
        } else {
            for i in 0..self.width {
                bits.push(reg.bit_ids[self.offset + i]);
            }
        }

        bits
    }

    /// Returns the bits associated with the field, wrapped in a BitCollection
    pub fn bits<'a>(&self, dut: &'a MutexGuard<Dut>) -> BitCollection<'a> {
        let bit_ids = self.bit_ids(dut);
        BitCollection::for_field(&bit_ids, self.reg_id, &self.name, dut)
    }

    /// Applies the given reset type, if the field doesn't have a reset defined with
    /// the given name then no action will be taken
    pub fn reset(&self, name: &str, dut: &MutexGuard<Dut>) {
        let r = self.resets.get(name);
        let bit_ids = self.bit_ids(dut);
        if r.is_some() {
            let rst = r.unwrap();
            // Sorry for the duplication, need to learn how to handle loops with an optional
            // parameter properly in Rust - ginty
            if rst.mask.is_some() {
                let mut bytes = rst.value.to_bytes_be();
                let mut byte = bytes.pop().unwrap();
                let mut mask_bytes = rst.value.to_bytes_be();
                let mut mask_byte = bytes.pop().unwrap();
                for i in 0..self.width {
                    let state = (byte >> i % 8) & 1;
                    let mask = (mask_byte >> i % 8) & 1;
                    if mask == 1 {
                        // Think its OK to panic here if this get_bit doesn't return something, things
                        // will have gone seriously wrong somewhere
                        dut.get_bit(bit_ids[i]).unwrap().reset(state);
                    } else {
                        dut.get_bit(bit_ids[i]).unwrap().reset(UNDEFINED);
                    }
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
            } else {
                let mut bytes = rst.value.to_bytes_be();
                let mut byte = bytes.pop().unwrap();
                for i in 0..self.width {
                    let state = (byte >> i % 8) & 1;
                    // Think its OK to panic here if this get_bit doesn't return something, things
                    // will have gone seriously wrong somewhere
                    dut.get_bit(bit_ids[i]).unwrap().reset(state);
                    if i % 8 == 7 {
                        match bytes.pop() {
                            Some(x) => byte = x,
                            None => byte = 0,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
/// A lightweight version of a Field that is returned by the my_reg.fields() iterator,
/// and which is also used to represent gaps in the register (when spacer = true).
pub struct SummaryField {
    pub reg_id: usize,
    pub name: String,
    pub offset: usize,
    /// Width of the field in bits.
    pub width: usize,
    pub access: AccessType,
    pub spacer: bool,
}

impl SummaryField {
    /// Returns the bits associated with the field, wrapped in a BitCollection
    pub fn bits<'a>(&self, dut: &'a MutexGuard<Dut>) -> BitCollection<'a> {
        let mut bit_ids: Vec<usize> = Vec::new();
        let reg = dut.get_register(self.reg_id).unwrap();

        for i in 0..self.width {
            bit_ids.push(reg.bit_ids[self.offset + i]);
        }

        BitCollection::for_field(&bit_ids, self.reg_id, &self.name, dut)
    }
}

#[derive(Debug)]
pub struct Reset {
    pub value: BigUint,
    pub mask: Option<BigUint>,
}

#[derive(Debug)]
pub struct EnumeratedValue {
    pub name: String,
    pub description: String,
    //pub usage: Usage,
    pub value: BigUint,
}
