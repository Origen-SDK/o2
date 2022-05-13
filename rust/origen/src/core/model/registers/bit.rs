use super::AccessType;
use super::AccessType::Unimplemented;
use crate::Result;
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

// State values for common initialization cases
pub const ZERO: u8 = 0;
pub const ONE: u8 = 1;
pub const UNDEFINED: u8 = 0b10;

#[derive(Debug, Clone, PartialEq)]
pub struct Overlay {
    pub label: Option<String>,
    pub symbol: Option<String>,
    pub persistent: bool,
}

impl fmt::Display for Overlay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}Overlay {}{}",
            {
                if self.persistent {
                    "Persistent "
                } else {
                    ""
                }
            },
            match &self.label {
                Some(l) => format!("'{}'", l),
                None => "<Anonymous Overlay>".to_string(),
            },
            match &self.symbol {
                Some(s) => format!("(Symbol '{}'", s),
                None => "".to_string(),
            },
        )
    }
}

impl Overlay {
    pub fn new(label: Option<String>, symbol: Option<String>, persistent: bool) -> Self {
        Self {
            label,
            symbol,
            persistent,
        }
    }
}

// TODO: Would one RwLock wrapping a BitInner struct instantiate faster?
#[derive(Debug)]
pub struct Bit {
    pub id: usize,
    pub register_id: usize,
    pub overlay: RwLock<Option<Overlay>>,
    pub overlay_snapshots: RwLock<HashMap<String, Option<Overlay>>>,
    /// The individual bits mean the following:
    /// 0 - Data value
    /// 1 - Value is X
    /// 2 - Value is Z
    /// 3 - Bit is to be verified
    /// 4 - Bit is to be captured
    /// 5 - Modified, sets if bits \[2:0\ have been changed since the last reset
    pub state: RwLock<u8>,
    /// The state we think the device has, only bits \[2:0\] are applicable.
    /// This is updated by resetting the register or executing a transaction.
    pub device_state: RwLock<u8>,
    /// The state of the bit at the last reset
    pub state_snapshots: RwLock<HashMap<String, u8>>,
    pub access: AccessType,
    pub position: usize,
}

impl Bit {
    pub fn snapshot(&self, name: &str) {
        let state = *self.state.read().unwrap();
        let mut state_snapshots = self.state_snapshots.write().unwrap();
        let mut overlay_snapshots = self.overlay_snapshots.write().unwrap();
        state_snapshots.insert(name.to_string(), state);
        overlay_snapshots.insert(name.to_string(), self.overlay.read().unwrap().clone());
    }

    pub fn is_changed(&self, name: &str) -> Result<bool> {
        let state = *self.state.read().unwrap();
        match self.state_snapshots.read().unwrap().get(name) {
            None => {
                bail!("No snapshot named '{}' has been taken", name)
            }
            Some(x) => {
                if *x != state {
                    return Ok(true);
                }
            }
        };
        match self.overlay_snapshots.read().unwrap().get(name) {
            None => {
                bail!("No snapshot named '{}' has been taken", name)
            }
            Some(x) => {
                if *x != *self.overlay.read().unwrap() {
                    return Ok(true);
                }
            }
        };
        Ok(false)
    }

    pub fn rollback(&self, name: &str) -> Result<()> {
        match self.state_snapshots.read().unwrap().get(name) {
            None => {
                bail!("No snapshot named '{}' has been taken", name)
            }
            Some(x) => {
                let mut state = self.state.write().unwrap();
                *state = *x;
            }
        };
        match self.overlay_snapshots.read().unwrap().get(name) {
            None => {
                bail!("No snapshot named '{}' has been taken", name)
            }
            Some(x) => {
                let mut ovl = self.overlay.write().unwrap();
                *ovl = x.clone();
            }
        };
        Ok(())
    }

    /// Copies the state (data and flags) and overlay attributes of the given bit to self
    pub fn copy_state(&self, source: &Bit) {
        let mut state = self.state.write().unwrap();
        *state = *source.state.read().unwrap();
        let mut overlay = self.overlay.write().unwrap();
        *overlay = source.overlay.read().unwrap().clone();
    }

    pub fn clear_flags(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val & 0b0010_0111;
    }

    pub fn clear_verify_flag(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val & 0b1111_0111;
    }

    pub fn capture(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val | 0b1_0000;
    }

    pub fn clear_capture(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val & 0b0_1111;
    }

    pub fn clear_persistent_overlay(&self) {
        let mut overlay = self.overlay.write().unwrap();
        *overlay = None;
    }

    pub fn clear_nonpersistent_overlay(&self) -> bool {
        let mut overlay = self.overlay.write().unwrap();
        if let Some(o) = &*overlay {
            if !o.persistent {
                *overlay = None;
                return true;
            }
        }
        return false;
    }

    /// Sets the bit's data value to X
    pub fn set_undefined(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val | 0b0010_0010;
    }

    /// Sets the bit's data value to Z
    pub fn set_hiz(&self) {
        let state_val;
        {
            state_val = *self.state.read().unwrap();
        }
        let mut state = self.state.write().unwrap();
        *state = state_val | 0b0010_0100;
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

    pub fn is_to_be_verified(&self) -> bool {
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

    pub fn is_modified_since_reset(&self) -> bool {
        *self.state.read().unwrap() & 0b10_0000 != 0
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

    pub fn verify(&self) -> Result<()> {
        if self.has_known_value() {
            let mut state = self.state.write().unwrap();
            *state = *state | 0b1000;
            Ok(())
        } else {
            bail!(
                "Attempt to verify a bit which has an undefined data value, bit state is: {}",
                self.state_char()
            );
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
            bail!(
                "Bit data value is unknown, bit state is: {}",
                self.state_char()
            );
        }
    }

    pub fn set_data(&self, val: u8) {
        // Let's make set_data ignore bit behaviour and can introduce an additional
        // behavioral-aware method in future
        //if self.is_writeable() {
        if !self.access.is_unimplemented() {
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
        *state = state_val | (val & 0b1) | 0b0010_0000;
    }

    pub fn get_overlay(&self) -> Option<Overlay> {
        match &*self.overlay.read().unwrap() {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn set_overlay(&self, label: Option<String>, sym: Option<String>, persistent: bool) {
        let mut overlay = self.overlay.write().unwrap();
        *overlay = Some(Overlay::new(label, sym, persistent));
    }

    pub fn verify_enable_flag(&self) -> u8 {
        if self.is_to_be_verified() {
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
        let mut device_state = self.device_state.write().unwrap();
        *device_state = val;
    }
}
