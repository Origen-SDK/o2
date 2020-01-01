//use super::pin::{PinActions};
//use crate::error::Error;
//use super::super::super::super::DUT;
use super::super::pins::{Endianness};
use super::super::Model;
use super::pin::{PinActions};
use crate::error::Error;
use super::pin_collection::PinCollection;

//use crate::core::model::Model;


// We'll maintain both the pin_ids which the group was built with, but we'll also maintain the list
// of physical IDs. Even though we can resolve this later, most operations wil 
#[derive(Debug, Clone)]
pub struct PinGroup {
    pub id: String,
    pub path: String,
    pub pin_ids: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
}

impl PinGroup {
    pub fn new(id: String, path: String, pins: Vec<String>, endianness: Option<Endianness>) -> PinGroup {
        return PinGroup {
            id: String::from(id),
            path: String::from(path),
            pin_ids: match endianness {
              Some(e) => {
                match e {
                  Endianness::LittleEndian => pins,
                  Endianness::BigEndian => {
                    let mut _pins = pins.clone();
                    _pins.reverse();
                    _pins
                  }
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
        return self.pin_ids.len();
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
  pub fn get_pin_group_data(&self, id: &str) -> u32 {
    let pin_ids = &self.get_pin_group(id).unwrap().pin_ids;
    self.get_pin_data(pin_ids)
  }

  pub fn get_pin_group_reset_data(&self, id: &str) -> u32 {
    let pin_ids = &self.get_pin_group(id).unwrap().pin_ids;
    self.get_pin_reset_data(&pin_ids)
  }

  pub fn reset_pin_group(&mut self, id: &str) -> Result<(), Error> {
    let pin_ids = self.get_pin_group(id).unwrap().pin_ids.clone();
    self.reset_pin_ids(&pin_ids)
  }

  pub fn set_pin_group_data(&mut self, id: &str, data: u32) -> Result<(), Error> {
    let grp = self.get_pin_group(id).unwrap();
    let m = grp.mask;
    let pin_ids = grp.pin_ids.clone();
    self.set_pin_data(&pin_ids, data, m)
  }

  pub fn resolve_pin_group_ids(&mut self, id: &str) -> Result<Vec<String>, Error> {
    let pin_ids = self.get_pin_group(id).unwrap().pin_ids.clone();
    self.resolve_pin_ids(&pin_ids)
  }

  /// Returns the pin actions as a string.
  /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the 
  ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
  pub fn get_pin_group_actions(&mut self, id: &str) -> Result<String, Error> {
    let pin_ids = self.get_pin_group(id).unwrap().pin_ids.clone();
    self.get_pin_actions(&pin_ids)
  }

  pub fn get_pin_group_reset_actions(&mut self, id: &str) -> Result<String, Error> {
    let pin_ids = self.get_pin_group(id).unwrap().pin_ids.clone();
    self.get_pin_reset_actions(&pin_ids)
  }

  pub fn set_pin_group_actions(&mut self, id: &str, action: PinActions, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
    let pin_ids = self.get_pin_group(id).unwrap().pin_ids.clone();
    self.set_pin_actions(&pin_ids, action, data, mask)
  }

  pub fn drive_pin_group(&mut self, group_id: &str, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
    return self.set_pin_group_actions(group_id, PinActions::Drive, data, mask);
  }

  pub fn verify_pin_group(&mut self, group_id: &str, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
    return self.set_pin_group_actions(group_id, PinActions::Verify, data, mask);
  }

  pub fn capture_pin_group(&mut self, group_id: &str, mask: Option<usize>) -> Result<(), Error> {
    return self.set_pin_group_actions(group_id, PinActions::Capture, Option::None, mask);
  }

  pub fn highz_pin_group(&mut self, group_id: &str, mask: Option<usize>) -> Result<(), Error> {
    return self.set_pin_group_actions(group_id, PinActions::HighZ, Option::None, mask);
  }

  // Assume the pin group is properly defined (that is, not pin duplicates and all pins exists. If the pin group exists, these should both be met)
  pub fn slice_pin_group(&mut self, id: &str, start_idx: usize, stop_idx: usize, step_size: usize) -> Result<PinCollection, Error> {
    if let Some(p) = self.get_pin_group(id) {
        let ids = &p.pin_ids;
        let mut sliced_ids: Vec<String> = vec!();

        for i in (start_idx..=stop_idx).step_by(step_size) {
            if i >= ids.len() {
                return Err(Error::new(&format!("Index {} exceeds available pins in group {} (length: {})", i, id, ids.len())));
            }
            let p = ids[i].clone();
            sliced_ids.push(p);
        }
        Ok(PinCollection::new(self.id, &self.name, &sliced_ids, Option::None))
        //Ok(PinCollection::new(&self.parent_path, &sliced_ids, Option::None))
    } else {
        Err(Error::new(&format!("Could not slice pin group {} because it doesn't exists!", id)))
    }
  }

  pub fn set_pin_group_nonsticky_mask(&mut self, id: &str, mask: usize) -> Result<(), Error> {
      let grp = self._get_mut_pin_group(id)?;
      grp.mask = Some(mask);
      Ok(())
  }
}
