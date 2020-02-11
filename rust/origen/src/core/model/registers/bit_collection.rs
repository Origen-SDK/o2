use super::register::Field;
use super::{Bit, Register};
use crate::{Dut, Error, Result};
use num_bigint::BigUint;
use regex::Regex;
use std::sync::MutexGuard;

const DONT_CARE_CHAR: &str = "X";
const OVERLAY_CHAR: &str = "V";
const STORE_CHAR: &str = "S";
const UNKNOWN_CHAR: &str = "?";

#[derive(Debug, Clone)]
pub struct BitCollection<'a> {
    /// Optionally contains the ID of the reg that owns the bits
    reg_id: Option<usize>,
    /// Optionally contains the name of the field that owns the bits
    field: Option<String>,
    /// When true the BitCollection contains all bits of the register defined
    /// by reg_id
    whole_reg: bool,
    /// When true the BitCollection contains all bits of the field defined
    /// by field
    whole_field: bool,
    pub bits: Vec<&'a Bit>,
    /// Iterator index and vars
    i: usize,
    shift_left: bool,
    shift_logical: bool,
}

impl<'a> Default for BitCollection<'a> {
    fn default() -> BitCollection<'a> {
        BitCollection {
            reg_id: None,
            field: None,
            whole_reg: false,
            whole_field: false,
            bits: Vec::new(),
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }
}

impl<'a> Iterator for BitCollection<'a> {
    type Item = &'a Bit;

    fn next(&mut self) -> Option<&'a Bit> {
        if self.i < self.len() {
            let bit;
            if self.shift_left {
                bit = self.bits[self.len() - self.i - 1];
            } else {
                bit = self.bits[self.i];
            }
            self.i += 1;
            Some(bit)
        } else {
            None
        }
    }
}

impl<'a> BitCollection<'a> {
    /// Creates a BitCollection from the given collection of bit IDs.
    /// The resultant collection can not be associated back to a register or field.
    /// Use the methods <reg>.bits() and <field>.bits() to create BitCollections with the necessary
    /// metadata to associate with the parent object.
    pub fn for_bit_ids(ids: &Vec<usize>, dut: &'a MutexGuard<'a, Dut>) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for id in ids {
            bits.push(dut.get_bit(*id).unwrap());
        }

        BitCollection {
            reg_id: None,
            field: None,
            whole_reg: false,
            whole_field: false,
            bits: bits,
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }

    /// Creates a BitCollection for the given register, normally this would not be called directly
    /// and would instead be called via <reg>.bits()
    pub fn for_register(reg: &Register, dut: &'a MutexGuard<'a, Dut>) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for id in &reg.bit_ids {
            bits.push(dut.get_bit(*id).unwrap());
        }

        BitCollection {
            reg_id: Some(reg.id),
            field: None,
            whole_reg: true,
            whole_field: false,
            bits: bits,
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }

    /// Creates a BitCollection for the given register field, normally this would not be called directly
    /// and would instead be called via <reg>.bits()
    pub fn for_field(
        ids: &Vec<usize>,
        reg_id: usize,
        name: &str,
        dut: &'a MutexGuard<'a, Dut>,
    ) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for id in ids {
            bits.push(dut.get_bit(*id).unwrap());
        }

        BitCollection {
            reg_id: Some(reg_id),
            field: Some(name.to_string()),
            whole_reg: false,
            whole_field: true,
            bits: bits,
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }

    pub fn set_data(&self, value: BigUint) {
        let mut bytes = value.to_bytes_be();
        let mut byte = bytes.pop().unwrap();

        for (i, &bit) in self.bits.iter().enumerate() {
            bit.set_data(byte >> i % 8);
            if i % 8 == 7 {
                match bytes.pop() {
                    Some(x) => byte = x,
                    None => byte = 0,
                }
            }
        }
    }

    /// Returns the data value of the BitCollection. This will return an error if
    /// any of the bits are undefined (X or Z).
    pub fn data(&self) -> Result<BigUint> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte: u8 = 0;
        for (i, &bit) in self.bits.iter().enumerate() {
            byte = byte | bit.data()? << i % 8;
            if i % 8 == 7 {
                bytes.push(byte);
                byte = 0;
            }
        }
        if self.bits.len() % 8 != 0 {
            bytes.push(byte);
        }
        Ok(BigUint::from_bytes_le(&bytes))
    }

    /// Returns the overlay value of the BitCollection. This will return an error if
    /// not all bits return the same value.
    pub fn get_overlay(&self) -> Result<Option<String>> {
        let val = self.bits[0].get_overlay();
        if !self.bits.iter().all(|&bit| bit.get_overlay() == val) {
            Err(Error::new(
                "The bits in the collection have different overlay values",
            ))
        } else {
            Ok(val)
        }
    }

    /// Set the overlay value of the BitCollection.
    pub fn set_overlay(&self, val: Option<&str>) -> &BitCollection {
        for &bit in self.bits.iter() {
            bit.set_overlay(val);
        }
        self
    }

    /// Returns true if no contained bits are in X or Z state
    pub fn has_known_value(&self) -> bool {
        self.bits.iter().all(|bit| bit.has_known_value())
    }

    /// Returns a new BitCollection containing the subset of bits within the given range
    pub fn range(&self, max: usize, min: usize) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for i in min..max + 1 {
            bits.push(self.bits[i]);
        }

        BitCollection {
            reg_id: self.reg_id,
            field: self.field.clone(),
            whole_reg: self.whole_reg && bits.len() == self.bits.len(),
            whole_field: self.whole_field && bits.len() == self.bits.len(),
            bits: bits,
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }

    /// Returns true if any bits in the collection has their read flag set
    pub fn is_to_be_read(&self) -> bool {
        self.bits.iter().any(|bit| bit.is_to_be_read())
    }

    /// Returns true if any bits in the collection has their capture flag set
    pub fn is_to_be_captured(&self) -> bool {
        self.bits.iter().any(|bit| bit.is_to_be_captured())
    }

    /// Returns true if any bits in the collection has an overlay set
    pub fn has_overlay(&self) -> bool {
        self.bits.iter().any(|bit| bit.has_overlay())
    }

    /// Returns true if any bits in the collection is writeable
    pub fn is_writeable(&self) -> bool {
        self.bits.iter().any(|bit| bit.is_writeable())
    }

    pub fn is_writable(&self) -> bool {
        self.is_writeable()
    }

    /// Returns true if any bits in the collection is readable
    pub fn is_readable(&self) -> bool {
        self.bits.iter().any(|bit| bit.is_readable())
    }

    pub fn is_update_required(&self) -> bool {
        self.bits.iter().any(|bit| bit.is_update_required())
    }

    /// Set the collection's device_state field to be the same as its current data state
    pub fn update_device_state(&self) -> Result<&BitCollection> {
        for &bit in self.bits.iter() {
            bit.update_device_state()?;
        }
        Ok(self)
    }

    pub fn clear_flags(&self) -> &BitCollection {
        for &bit in self.bits.iter() {
            bit.clear_flags();
        }
        self
    }

    pub fn capture(&self) -> &BitCollection {
        for &bit in self.bits.iter() {
            bit.capture();
        }
        self
    }

    pub fn set_undefined(&self) -> &BitCollection {
        for &bit in self.bits.iter() {
            bit.set_undefined();
        }
        self
    }

    /// Resets the bits if the collection is for a whole bit field or register, otherwise
    /// an error will be raised
    pub fn reset(&self, name: &str, dut: &'a MutexGuard<'a, Dut>) -> Result<&'a BitCollection> {
        if self.whole_reg || self.whole_field {
            if self.whole_reg {
                self.reg(dut)?.reset(name, dut);
            } else {
                self.field(dut)?.reset(name, dut);
            }
            Ok(self)
        } else {
            Err(Error::new(
                "Reset cannot be called on an ad-hoc BitCollection, only on a Register or a named Bit Field"
            ))
        }
    }

    //pub fn read(&self, dut: &'a MutexGuard<'a, Dut>) -> Result<&'a BitCollection> {
    pub fn read(&self) -> Result<&BitCollection> {
        for &bit in self.bits.iter() {
            bit.read()?;
        }
        // TODO: Record the read in the AST here
        Ok(self)
    }

    /// Returns the Register object associated with the BitCollection. Note that this will
    /// return the reg even if the BitCollection only contains a subset of the register's bits
    pub fn reg(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a Register> {
        match self.reg_id {
            Some(x) => dut.get_register(x),
            None => Err(Error::new(
                "Tried to reference the Register object from a BitCollection with no reg_id",
            )),
        }
    }

    /// Returns the bit Field object associated with the BitCollection. Note that this will
    /// return the Field even if the BitCollection only contains a subset of the field's bits
    pub fn field(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a Field> {
        match &self.field {
            Some(x) => Ok(&self.reg(dut)?.fields[x]),
            None => Err(Error::new(
                "Tried to reference the Field object from a BitCollection with no field data",
            )),
        }
    }

    pub fn shift_left(&self, shift_in: u8) -> Result<u8> {
        let mut v1 = shift_in & 0x1;
        let mut v2: u8;
        for &bit in self.bits.iter() {
            v2 = bit.data()? & 0x1;
            bit.set_data(v1);
            v1 = v2;
        }
        Ok(v1)
    }

    pub fn shift_right(&self, shift_in: u8) -> Result<u8> {
        let mut v1 = shift_in & 0x1;
        let mut v2: u8;
        for &bit in self.bits.iter().rev() {
            v2 = bit.data()? & 0x1;
            bit.set_data(v1);
            v1 = v2;
        }
        Ok(v1)
    }

    pub fn shift_out_left(&self) -> BitCollection {
        let mut bc = self.clone();
        bc.i = 0;
        bc.shift_left = true;
        bc.shift_logical = false;
        bc
    }

    pub fn shift_out_right(&self) -> BitCollection {
        let mut bc = self.clone();
        bc.i = 0;
        bc.shift_left = false;
        bc.shift_logical = false;
        bc
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn read_enables(&self) -> BigUint {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        for (i, &bit) in self.bits.iter().enumerate() {
            byte = byte | bit.read_enable_flag() << i % 8;
            if i % 8 == 7 {
                bytes.push(byte);
                byte = 0;
            }
        }
        if self.bits.len() % 8 != 0 {
            bytes.push(byte);
        }
        BigUint::from_bytes_le(&bytes)
    }

    pub fn capture_enables(&self) -> BigUint {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        for (i, &bit) in self.bits.iter().enumerate() {
            byte = byte | bit.capture_enable_flag() << i % 8;
            if i % 8 == 7 {
                bytes.push(byte);
                byte = 0;
            }
        }
        if self.bits.len() % 8 != 0 {
            bytes.push(byte);
        }
        BigUint::from_bytes_le(&bytes)
    }

    pub fn overlay_enables(&self) -> BigUint {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        for (i, &bit) in self.bits.iter().enumerate() {
            byte = byte | bit.overlay_enable_flag() << i % 8;
            if i % 8 == 7 {
                bytes.push(byte);
                byte = 0;
            }
        }
        if self.bits.len() % 8 != 0 {
            bytes.push(byte);
        }
        BigUint::from_bytes_le(&bytes)
    }

    pub fn status_str(&mut self, operation: &str) -> Result<String> {
        let mut ss = "".to_string();
        if operation == "read" || operation == "r" {
            for bit in self.shift_out_left() {
                if bit.is_to_be_captured() {
                    ss += STORE_CHAR;
                } else if bit.is_to_be_read() {
                    if bit.has_overlay() {
                        //&& options[:mark_overlays]
                        ss += OVERLAY_CHAR
                    } else {
                        if bit.has_known_value() {
                            if bit.data().unwrap() == 0 {
                                ss += "0";
                            } else {
                                ss += "1";
                            }
                        } else {
                            ss += UNKNOWN_CHAR;
                        }
                    }
                } else {
                    ss += DONT_CARE_CHAR;
                }
            }
        } else if operation == "write" || operation == "w" {
            for bit in self.shift_out_left() {
                if bit.has_overlay() {
                    //&& options[:mark_overlays]
                    ss += OVERLAY_CHAR;
                } else {
                    if bit.has_known_value() {
                        if bit.data().unwrap() == 0 {
                            ss += "0";
                        } else {
                            ss += "1";
                        }
                    } else {
                        ss += UNKNOWN_CHAR;
                    }
                }
            }
        } else {
            return Err(Error::new(&format!(
                "Unknown operation argument '{}', must be \"read\" or \"write\"",
                operation
            )));
        }
        Ok(BitCollection::make_hex_like(
            &ss,
            (self.len() as f64 / 4.0).ceil() as usize,
        ))
    }

    // Converts a binary-like representation of a data value into a hex-like version.
    // e.g. input  => 010S0011SSSS0110   (where S, X or V represent store, don't care or overlay)
    //      output => [010s]3S6    (i.e. nibbles that are not all of the same type are expanded)
    fn make_hex_like(regval: &str, size_in_nibbles: usize) -> String {
        let mut outstr = "".to_string();
        let mut re = "^(.?.?.?.)".to_string();
        for _i in 0..size_in_nibbles - 1 {
            re += "(....)";
        }
        re += "$";
        let regex = Regex::new(&format!(r"{}", re)).unwrap();

        let captures = regex.captures(regval).unwrap();

        let mut nibbles: Vec<&str> = Vec::new();
        for i in 0..size_in_nibbles {
            // now grouped by nibble
            nibbles.push(&captures[i + 1]);
        }

        let regex = Regex::new(&format!(
            r"[{}{}{}{}]",
            UNKNOWN_CHAR, DONT_CARE_CHAR, STORE_CHAR, OVERLAY_CHAR
        ))
        .unwrap();

        for nibble in nibbles {
            // If contains any special chars...
            if regex.is_match(nibble) {
                let c1 = nibble.chars().next().unwrap();
                // If all the same...
                if nibble.chars().count() == 4 && nibble.chars().all(|c2| c1 == c2) {
                    outstr += &format!("{}", c1);
                // Otherwise present this nibble in 'binary' format
                } else {
                    outstr += &format!("[{}]", nibble.to_ascii_lowercase());
                }
            // Otherwise if all 1s and 0s...
            } else {
                let n: u32 = u32::from_str_radix(nibble, 2).unwrap();
                outstr += &format!("{:X?}", n);
            }
        }
        outstr
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::registers::{Bit, BitCollection};
    use crate::{dut, Dut};
    use num_bigint::ToBigUint;
    use std::sync::MutexGuard;

    fn make_bit_collection<'a>(size: usize, dut: &'a mut MutexGuard<Dut>) -> BitCollection<'a> {
        let mut bit_ids: Vec<usize> = Vec::new();
        for _i in 0..size {
            bit_ids.push(dut.create_test_bit());
        }

        let mut bits: Vec<&Bit> = Vec::new();
        for id in bit_ids {
            bits.push(dut.get_bit(id).unwrap());
        }

        BitCollection {
            reg_id: None,
            field: None,
            whole_reg: false,
            whole_field: false,
            bits: bits,
            i: 0,
            shift_left: false,
            shift_logical: false,
        }
    }

    #[test]
    fn data_method_works() {
        let mut dut = dut();
        let bc = make_bit_collection(16, &mut dut);

        assert_eq!(bc.data().unwrap(), 0.to_biguint().unwrap());
    }

    #[test]
    fn set_data_method_works() {
        let mut dut = dut();
        let bc = make_bit_collection(16, &mut dut);

        bc.set_data(0.to_biguint().unwrap());
        assert_eq!(bc.data().unwrap(), 0.to_biguint().unwrap());
        bc.set_data(0xFFFF.to_biguint().unwrap());
        assert_eq!(bc.data().unwrap(), 0xFFFF.to_biguint().unwrap());
        bc.set_data(0x1234.to_biguint().unwrap());
        assert_eq!(bc.data().unwrap(), 0x1234.to_biguint().unwrap());
    }

    #[test]
    fn range_method_works() {
        let mut dut = dut();
        let bc = make_bit_collection(16, &mut dut);

        bc.set_data(0x1234.to_biguint().unwrap());
        assert_eq!(bc.data().unwrap(), 0x1234.to_biguint().unwrap());
        assert_eq!(bc.range(3, 0).data().unwrap(), 0x4.to_biguint().unwrap());
        assert_eq!(bc.range(7, 4).data().unwrap(), 0x3.to_biguint().unwrap());
        assert_eq!(bc.range(15, 8).data().unwrap(), 0x12.to_biguint().unwrap());

        let bc = make_bit_collection(8, &mut dut);
        bc.set_data(0x1F.to_biguint().unwrap());
        assert_eq!(bc.range(4, 0).data().unwrap(), 0x1F.to_biguint().unwrap());
        assert_eq!(bc.range(7, 4).data().unwrap(), 0x1.to_biguint().unwrap());
    }
}
