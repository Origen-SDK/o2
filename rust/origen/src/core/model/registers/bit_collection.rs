use super::Bit;
use crate::{Dut, Result};
use num_bigint::BigUint;
use std::sync::MutexGuard;

#[derive(Debug)]
pub struct BitCollection<'a> {
    /// When true the BitCollection contains an entire register's worth of bits
    whole: bool,
    bits: Vec<&'a Bit>,
    /// Iterator index
    i: usize,
}

impl<'a> BitCollection<'a> {
    pub fn for_bit_ids(ids: Vec<usize>, dut: &'a MutexGuard<'a, Dut>) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for id in ids {
            bits.push(dut.get_bit(id).unwrap());
        }

        BitCollection {
            whole: false,
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
                    None => return,
                }
            }
        }
    }

    pub fn data(&self) -> Result<BigUint> {
        let mut bytes: Vec<u8> = Vec::new();

        //self.bits.iter().all(|bit| bit.has_known_value())
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

    /// Returns true if no contained bits are in X or Z state
    pub fn range(&self, max: usize, min: usize) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for i in min..max {
            bits.push(self.bits[i]);
        }

        BitCollection {
            whole: false,
            bits: bits,
            i: 0,
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
            whole: false,
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
    }
}
