use super::AccessType;
use super::AccessType::Unimplemented;
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::RwLock;

// State values for common initialization cases
pub const ZERO: u8 = 0;
pub const ONE: u8 = 1;
pub const UNDEFINED: u8 = 0b10;

// TODO: Would one RwLock wrapping a BitInner struct instantiate faster?
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
    pub state: RwLock<u8>,
    /// The state we think the device has, only bits [2:0] are applicable.
    /// This is updated by reseting the register or executing a transaction.
    pub device_state: RwLock<u8>,
    /// The state of the bit at the last reset
    pub reset_state: RwLock<u8>,
    pub state_snapshots: RwLock<HashMap<String, u8>>,
    pub access: AccessType,
}

impl Bit {
    /// Copies the state (data and flags) and overlay attributes of the given bit to self
    pub fn copy_state(&self, source: &Bit) {
        let mut state = self.state.write().unwrap();
        *state = *source.state.read().unwrap();
        let mut overlay = self.overlay.write().unwrap();
        match &*source.overlay.read().unwrap() {
            Some(x) => *overlay = Some(x.to_string()),
            None => *overlay = None,
        }
    }

    pub fn clear_flags(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val & 0b111;
    }

    pub fn capture(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val | 0b1_0000;
    }

    /// Sets the bit's data value to X
    pub fn set_undefined(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val | 0b10;
    }

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
        (*self.overlay.read().unwrap()).is_some()
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

    pub fn read(&self) -> Result<()> {
        if self.has_known_value() {
            let mut state = self.state.write().unwrap();
            *state = *state | 0b1000;
            Ok(())
        } else {
            return Err(Error::new(&format!(
                "Attempt to read a bit which has an undefined data value, bit state is: {}",
                self.state_char()
            )));
        }
    }

    /// Returns true if the current bit state differs from the device state.
    /// Note that for the purposes of this comparison, X and a 1/0 are considered different.
    pub fn is_update_required(&self) -> bool {
        *self.state.read().unwrap() & 0b111 != *self.device_state.read().unwrap() & 0b111
    }

    pub fn update_device_state(&self) -> Result<()> {
        let s = self.state.read().unwrap();
        let mut d = self.device_state.write().unwrap();
        *d = *s;
        Ok(())
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

    pub fn get_overlay(&self) -> Option<String> {
        match &*self.overlay.read().unwrap() {
            Some(x) => Some(x.to_string()),
            None => None,
        }
    }

    pub fn set_overlay(&self, val: Option<&str>) {
        let mut overlay = self.overlay.write().unwrap();
        match val {
            Some(x) => *overlay = Some(x.to_string()),
            None => *overlay = None,
        }
    }

    pub fn read_enable_flag(&self) -> u8 {
        if self.is_to_be_read() {
            1
        } else {
            0
        }
    }

    pub fn capture_enable_flag(&self) -> u8 {
        if self.is_to_be_captured() {
            1
        } else {
            0
        }
    }

    pub fn overlay_enable_flag(&self) -> u8 {
        if self.has_overlay() {
            1
        } else {
            0
        }
    }

    /// Applies the given state value, making it the new reset baseline and
    /// the device_state
    pub fn reset(&self, val: u8) {
        let mut state = self.state.write().unwrap();
        *state = val;
        let mut reset_state = self.reset_state.write().unwrap();
        *reset_state = val;
        let mut device_state = self.device_state.write().unwrap();
        *device_state = val;
    }
}
