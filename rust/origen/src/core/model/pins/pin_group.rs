//use super::pin::{PinActions};
//use crate::error::Error;
//use super::super::super::super::DUT;
use super::super::pins::Endianness;
use super::super::Model;
use super::pin::PinActions;
use super::pin_collection::PinCollection;
use crate::error::Error;

//use crate::core::model::Model;

// We'll maintain both the pin_names which the group was built with, but we'll also maintain the list
// of physical names. Even though we can resolve this later, most operations wil
#[derive(Debug, Clone)]
pub struct PinGroup {
    pub name: String,
    pub path: String,
    pub pin_names: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
}

impl PinGroup {
    pub fn new(
        name: String,
        path: String,
        pins: Vec<String>,
        endianness: Option<Endianness>,
    ) -> PinGroup {
        return PinGroup {
            name: String::from(name),
            path: String::from(path),
            pin_names: match endianness {
                Some(e) => match e {
                    Endianness::LittleEndian => pins,
                    Endianness::BigEndian => {
                        let mut _pins = pins.clone();
                        _pins.reverse();
                        _pins
                    }
                },
                None => pins,
            },
            endianness: match endianness {
                Some(e) => e,
                None => Endianness::LittleEndian,
            },
            mask: Option::None,
        };
    }

    pub fn len(&self) -> usize {
        return self.pin_names.len();
    }

    pub fn is_little_endian(&self) -> bool {
        match self.endianness {
            Endianness::LittleEndian => true,
            Endianness::BigEndian => false,
        }
    }

    pub fn is_big_endian(&self) -> bool {
        return !self.is_little_endian();
    }
}

impl Model {
    pub fn get_pin_group_data(&self, name: &str) -> u32 {
        let pin_names = &self.get_pin_group(name).unwrap().pin_names;
        self.get_pin_data(pin_names)
    }

    pub fn get_pin_group_reset_data(&self, name: &str) -> u32 {
        let pin_names = &self.get_pin_group(name).unwrap().pin_names;
        self.get_pin_reset_data(&pin_names)
    }

    pub fn reset_pin_group(&mut self, name: &str) -> Result<(), Error> {
        let pin_names = self.get_pin_group(name).unwrap().pin_names.clone();
        self.reset_pin_names(&pin_names)
    }

    pub fn set_pin_group_data(&mut self, name: &str, data: u32) -> Result<(), Error> {
        let grp = self.get_pin_group(name).unwrap();
        let m = grp.mask;
        let pin_names = grp.pin_names.clone();
        self.set_pin_data(&pin_names, data, m)
    }

    pub fn resolve_pin_group_names(&mut self, name: &str) -> Result<Vec<String>, Error> {
        let pin_names = self.get_pin_group(name).unwrap().pin_names.clone();
        self.resolve_pin_names(&pin_names)
    }

    /// Returns the pin actions as a string.
    /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the
    ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
    pub fn get_pin_group_actions(&mut self, name: &str) -> Result<String, Error> {
        let pin_names = self.get_pin_group(name).unwrap().pin_names.clone();
        self.get_pin_actions(&pin_names)
    }

    pub fn get_pin_group_reset_actions(&mut self, name: &str) -> Result<String, Error> {
        let pin_names = self.get_pin_group(name).unwrap().pin_names.clone();
        self.get_pin_reset_actions(&pin_names)
    }

    pub fn set_pin_group_actions(
        &mut self,
        name: &str,
        action: PinActions,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        let pin_names = self.get_pin_group(name).unwrap().pin_names.clone();
        self.set_pin_actions(&pin_names, action, data, mask)
    }

    pub fn drive_pin_group(
        &mut self,
        group_name: &str,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        return self.set_pin_group_actions(group_name, PinActions::Drive, data, mask);
    }

    pub fn verify_pin_group(
        &mut self,
        group_name: &str,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        return self.set_pin_group_actions(group_name, PinActions::Verify, data, mask);
    }

    pub fn capture_pin_group(
        &mut self,
        group_name: &str,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        return self.set_pin_group_actions(group_name, PinActions::Capture, Option::None, mask);
    }

    pub fn highz_pin_group(&mut self, group_name: &str, mask: Option<usize>) -> Result<(), Error> {
        return self.set_pin_group_actions(group_name, PinActions::HighZ, Option::None, mask);
    }

    // Assume the pin group is properly defined (that is, not pin duplicates and all pins exists. If the pin group exists, these should both be met)
    pub fn slice_pin_group(
        &mut self,
        name: &str,
        start_idx: usize,
        stop_idx: usize,
        step_size: usize,
    ) -> Result<PinCollection, Error> {
        if let Some(p) = self.get_pin_group(name) {
            let names = &p.pin_names;
            let mut sliced_names: Vec<String> = vec![];

            for i in (start_idx..=stop_idx).step_by(step_size) {
                if i >= names.len() {
                    return Err(Error::new(&format!(
                        "Index {} exceeds available pins in group {} (length: {})",
                        i,
                        name,
                        names.len()
                    )));
                }
                let p = names[i].clone();
                sliced_names.push(p);
            }
            Ok(PinCollection::new(
                self.id,
                &self.name,
                &sliced_names,
                Option::None,
            ))
        } else {
            Err(Error::new(&format!(
                "Could not slice pin group {} because it doesn't exists!",
                name
            )))
        }
    }

    pub fn set_pin_group_nonsticky_mask(&mut self, name: &str, mask: usize) -> Result<(), Error> {
        let grp = self._get_mut_pin_group(name)?;
        grp.mask = Some(mask);
        Ok(())
    }
}
