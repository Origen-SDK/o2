use super::Bit;
use crate::{Dut, Result};
use num_bigint::BigUint;
use std::sync::{MutexGuard, RwLock};

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
        Ok(BigUint::from_bytes_be(&bytes))
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
    use std::cmp;
    use std::sync::MutexGuard;

    fn make_bits(number: usize) {
        for i in 0..number {
            dut().bits.push(Bit {
                overlay: RwLock::new(None),
                register_id: 0,
                state: RwLock::new(0),
                unimplemented: false,
            });
        }
    }

    fn make_bit_collection<'a>(size: usize, dut: &'a MutexGuard<Dut>) -> BitCollection<'a> {
        let mut bits: Vec<&Bit> = Vec::new();

        for i in 0..size {
            bits.push(dut.get_bit(i).unwrap());
        }

        BitCollection {
            whole: false,
            bits: bits,
            i: 0,
        }
    }

    #[test]
    fn data_method_works() {
        make_bits(128);
        let dut = dut();
        let bc = make_bit_collection(16, &dut);

        assert_eq!(bc.data().unwrap(), 0.to_biguint().unwrap());
        for i in 0..16 {
            dut.get_bit(i).unwrap().set_data(1);
        }
        assert_eq!(bc.data().unwrap(), 0xFF.to_biguint().unwrap());
    }
}
