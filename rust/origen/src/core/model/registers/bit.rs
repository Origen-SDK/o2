use super::AccessType;
use super::AccessType::Unimplemented;
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::RwLock;

// State values for common initialization cases
pub const ZERO: u8 = 0;
pub const ONE: u8 = 1;
pub const UNDEFINED: u8 = 0b10;

#[derive(Debug)]
pub struct Bit {
    pub register_id: usize,
    pub overlay: RwLock<Option<String>>,
    pub overlay_snapshots: RwLock<HashMap<String, Option<String>>>,
    /// The individual bits mean the following:
    /// 0 - Data value
    /// 1 - Value is X
    /// 2 - Value is Z
    /// 3 - Bit is to be read
    /// 4 - Bit is to be captured
    /// 5 - Bit has an overlay (defined by overlay str)
    pub state: RwLock<u8>,
    /// The state of the bit at the last reset
    pub reset_state: RwLock<u8>,
    pub state_snapshots: RwLock<HashMap<String, u8>>,
    pub access: AccessType,
}

impl Bit {
    /// Returns true if not in X or Z state
    pub fn has_known_value(&self) -> bool {
        self.access == Unimplemented || *self.state.read().unwrap() & 0b110 == 0
    }

    pub fn is_x(&self) -> bool {
        self.access != Unimplemented && *self.state.read().unwrap() & 0b10 != 0
    }

    pub fn is_z(&self) -> bool {
        self.access != Unimplemented && *self.state.read().unwrap() & 0b100 != 0
    }

    pub fn is_to_be_read(&self) -> bool {
        *self.state.read().unwrap() & 0b1000 != 0
    }

    pub fn is_to_be_captured(&self) -> bool {
        *self.state.read().unwrap() & 0b1_0000 != 0
    }

    pub fn has_overlay(&self) -> bool {
        *self.state.read().unwrap() & 0b10_0000 != 0
    }

    pub fn is_readable(&self) -> bool {
        self.access.is_readable()
    }

    pub fn is_writeable(&self) -> bool {
        self.access.is_writeable()
    }

    pub fn is_writable(&self) -> bool {
        self.access.is_writable()
    }

    pub fn state_char(&self) -> char {
        if self.has_known_value() {
            if *self.state.read().unwrap() & 0b1 == 0 {
                '0'
            } else {
                '1'
            }
        } else {
            if self.is_x() {
                'x'
            } else {
                'z'
            }
        }
    }

    pub fn data(&self) -> Result<u8> {
        if self.has_known_value() {
            Ok(*self.state.read().unwrap() & 0b1)
        } else {
            return Err(Error::new(&format!(
                "Bit data value is unknown, bit state is: {}",
                self.state_char()
            )));
        }
    }

    pub fn set_data(&self, val: u8) {
        if self.is_writeable() {
            self.force_data(val);
        }
    }

    /// Like set_data(), but will force the data value in the event of the bit being unimplemented or
    /// otherwise unwritable
    pub fn force_data(&self, val: u8) {
        let state_val;
        {
            // Clear X and Z flags
            state_val = *self.state.read().unwrap() & 0b1111_1000;
        }
        let mut state = self.state.write().unwrap();
        // Clear X and Z flags
        *state = state_val | (val & 0b1);
    }

    /// Clears all flags and applies the given data value and makes it the new reset baseline
    pub fn reset(&self, val: u8) {
        let mut state = self.state.write().unwrap();
        *state = val & 0b1;
        let mut reset_state = self.reset_state.write().unwrap();
        *reset_state = val & 0b1;
    }
}
