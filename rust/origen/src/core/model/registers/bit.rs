use crate::{Error, Result};
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct Bit {
    pub register_id: usize,
    pub overlay: RwLock<Option<String>>,
    /// The individual bits mean the following:
    /// 0 - Data value
    /// 1 - Value is X
    /// 2 - Value is Z
    /// 3 - Bit is to be read
    /// 4 - Bit is to be captured
    /// 5 - Bit has an overlay (defined by overlay str)
    pub state: RwLock<u8>,
    pub unimplemented: bool,
}

impl Bit {
    /// Returns true if not in X or Z state
    pub fn has_known_value(&self) -> bool {
        self.unimplemented || *self.state.read().unwrap() & 0b110 == 0
    }

    pub fn is_x(&self) -> bool {
        !self.unimplemented && *self.state.read().unwrap() & 0b10 != 0
    }

    pub fn is_z(&self) -> bool {
        !self.unimplemented && *self.state.read().unwrap() & 0b100 != 0
    }

    pub fn is_to_be_read(&self) -> bool {
        *self.state.read().unwrap() & 0b1000 != 0
    }

    pub fn is_to_be_captured(&self) -> bool {
        *self.state.read().unwrap() & 0b1_0000 != 0
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
        if !self.unimplemented {
            let state_val;
            {
                // Clear X and Z flags
                state_val = *self.state.read().unwrap() & 0b1111_1000;
            }
            let mut state = self.state.write().unwrap();
            // Clear X and Z flags
            *state = state_val | (val & 0b1);
        }
    }
}