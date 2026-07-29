use std::{time::{SystemTime, UNIX_EPOCH}, collections::HashMap};

use super::bin_array::BinArray;
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Numbers to be included
    pub include: BinArray,
    /// Numbers to be excluded
    pub exclude: BinArray,
    /// Numbers that have been manually assigned
    pub reserved: BinArray,
    pub references: HashMap<u32, u128>,
    pub exhausted: bool,
    pub increment: u32,
}

impl Item {
    pub fn new() -> Self {
        Self {
            include: BinArray::new(),
            exclude: BinArray::new(),
            reserved: BinArray::new(),
            references: HashMap::new(),
            exhausted: false,
            increment: 1
        }
    }

    pub fn next(&mut self, mut size: Option<u32>) -> Option<u32> {
        if size.is_none() {
            size = match self.increment {
                1 => None,
                _ => Some(self.increment)
            };
        }
        if !self.exhausted {
            loop {
                if let Some(n) = self.include.next(None, size) {
                    if self.is_valid(n) {
                        return Some(n);
                    }
                } else {
                    self.exhausted = true;
                    break;
                }
            }
        }
        // We can assume that any previous references were not excluded
        let size = size.unwrap_or(1);
        loop {
            // While self.oldest_reference() is not None and the oldest reference + size is not
            // in the exclude list, return the oldest reference
            if let Some(n) = self.oldest_reference() {
                for i in 0..size {
                    if !self.is_valid(n + i) {
                        break;
                    }
                }
                for i in 0..size {
                    self.record_reference(n + i);
                }
                return Some(n);
            } else {
                return None;
            }
        }
    }
    
    fn is_valid(&self, num: u32) -> bool {
        !self.exclude.contains(num) && !self.reserved.contains(num)
    }
    
    /// Records that the given number is being referenced now
    pub fn record_reference(&mut self, num: u32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        self.references.insert(num, now);
    }

    /// Returns the number that was referenced the longest time ago and updates its reference to now
    pub fn oldest_reference(&mut self) -> Option<u32> {
        if self.references.is_empty() {
            return None;
        }   
        // Find the entry with the smallest timestamp
        let (&oldest_num, _) = self.references
            .iter()
            .min_by_key(|&(_, ts)| ts)?;
        Some(oldest_num)
    }
    
    // Returns an iterator over all included numbers
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.include.iter().filter(move |&num| !self.exclude.contains(num) && !self.reserved.contains(num))
    }
}
