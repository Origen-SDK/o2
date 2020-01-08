use super::Bit;
use crate::Dut;
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

    /// Returns true if no contained bits are in X or Z state
    pub fn has_known_value(&self) -> bool {
        self.bits.iter().all(|bit| bit.has_known_value())
    }
}
