use super::{Bit, Register};
use super::register::Field;
use crate::{Dut, Result, Error};
use num_bigint::BigUint;
use std::sync::MutexGuard;

#[derive(Debug)]
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
    /// Iterator index
    i: usize,
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

    /// Resets the bits if the collection is for a whole bit field or register, otherwise
    /// an error will be raised
    pub fn reset(&self, name: &str, dut: &'a MutexGuard<'a, Dut>) -> Result<()> {
        if self.whole_reg || self.whole_field {
            if self.whole_reg {
                self.reg(dut)?.reset(name, dut);
            } else {
                self.field(dut)?.reset(name, dut);
            }
            Ok(())
        } else {
            Err(Error::new(
                "Reset cannot be called on an ad-hoc BitCollection, only on a Register or a named Bit Field"
            ))
        }
    }

    /// Returns the Register object associated with the BitCollection. Note that this will
    /// return the reg even if the BitCollection only contains a subset of the register's bits
    pub fn reg(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a Register> {
        match self.reg_id {
            Some(x) => dut.get_register(x),
            None => Err(Error::new("Tried to reference the Register object from a BitCollection with no reg_id")),
        }
    }
    
    /// Returns the bit Field object associated with the BitCollection. Note that this will
    /// return the Field even if the BitCollection only contains a subset of the field's bits
    pub fn field(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a Field> {
        match &self.field {
            Some(x) => Ok(&self.reg(dut)?.fields[x]),
            None => Err(Error::new("Tried to reference the Field object from a BitCollection with no field data")),
        }
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
