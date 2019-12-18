pub mod pin;
pub mod pin_group;
pub mod pin_collection;
use crate::error::Error;

use std::collections::HashMap;
//use pin_collection::{PinCollection};
use pin::{Pin, PinActions};
use pin_group::PinGroup;
use super::Model;

impl Model {
    pub fn add_pin(&mut self, id: &str, path: &str) -> Result<&mut Pin, Error> {
        let n = id;
        if self.pin(id).is_some() {
            return Err(Error::new(&format!("Can not add pin {} because it conflicts with a current pin or alias id!", id)))
        }
        let p = Pin::new(String::from(n), String::from(path));
        self.pins.insert(String::from(n), p);
        Ok(self.pins.get_mut(n).unwrap())
    }

    /// Returns the phyiscal pin, or None, if the pin doesn't exist.
    /// Implementation note: based on anecdotal evidence, physical pins generally have hardware-oriented pins, e.g.: PTA0, GPIO_A, etc.
    ///   The app will alias those to friendlier ids like swd_io, swdclk, which patterns and drivers will use.
    ///   So, I'd expect a hit in the alias HashMap more often than the in actual Pins', so check the alias first, then fall back to the pins.
    pub fn pin(&mut self, pin: &str) -> Option<&mut Pin> {
        let mut _p = pin;
        if let Some(p) = self.pin_aliases.get(_p) {
            _p = p;
        }
        if let Some(_pin) = self.pins.get_mut(_p) {
            Option::Some(_pin)
        } else {
            Option::None
        }
        //if let Some(p) = self.pins.get_mut(pin) {
        //  Ok(p)
        //} else {
        //  Err(Error::new(&format!("No pin {} available!", pin)))
        //}
    }

    pub fn get_pin(&self, pin: &str) -> Option<&Pin> {
        let mut _p = pin;
        if let Some(p) = self.pin_aliases.get(_p) {
            _p = p;
        }
        if let Some(_pin) = self.pins.get(_p) {
            Option::Some(_pin)
        } else {
            Option::None
        }
        //if let Some(p) = self.pins.get_mut(pin) {
        //  Ok(p)
        //} else {
        //  Err(Error::new(&format!("No pin {} available!", pin)))
        //}
    }

    //pub fn ids_for(&mut self, pin:)
    
    /// Returns a HashMap with all pin ids and, in the event of an alias,the pin id its aliased to.
    /// Todo: There's probably a more optimized way to do this since we're essentially just taking the pin alias hashmap
    ///       and merge it with pins, after mapping the value to the same key.
    pub fn pins(&mut self) -> HashMap<String, String> {
        let mut retn = HashMap::new();
        for (n, a) in self.pin_aliases.iter() {
            retn.insert(n.clone(), a.clone());
        }
        for (n, _p) in self.pins.iter() {
            retn.insert(n.clone(), n.clone());
        }
        return retn;
    }

    pub fn has_pin(&mut self, id_or_alias: &str) -> bool {
        return self.pin(id_or_alias).is_some();
    }

    pub fn number_of_pins(&mut self) -> usize {
        return self.pins.len();
    }

    pub fn add_pin_alias(&mut self, id: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if !self.pins.contains_key(id) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} doesn't exists!", id, alias, id)))
        }
        if self.pins.contains_key(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as alias {} already exists as a pin!", id, alias, alias)))
        }

        // Check if the alias already exists. If so, raise an exception, otherwise add the alias.
        if let Some(_id) = self.pin_aliases.get(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} is already aliased to {}", id, alias, _id, alias)))
        } else {
            self.pin_aliases.insert(String::from(alias), String::from(id));
            Ok(())
        }
    }

    pub fn group_pins(&mut self, id: &str, path: &str, pins: Vec<String>) -> Result<&mut PinGroup, Error> {
        let n = id;
        if self.pin_group(id).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias id!", id)))
        }
        let p = PinGroup::new(String::from(n), String::from(path), pins);
        self.pin_groups.insert(String::from(n), p);
        Ok(self.pin_groups.get_mut(n).unwrap())
    }

    pub fn pin_group(&mut self, id: &str) -> Option<&mut PinGroup> {
        let mut _n = id;
        if let Some(n) = self.pin_group_aliases.get(_n) {
            _n = n;
        }
        if let Some(_pin_group) = self.pin_groups.get_mut(_n) {
            Option::Some(_pin_group)
        } else {
            Option::None
        }
    }

    pub fn get_pin_group(&self, id: &str) -> Option<&PinGroup> {
        let mut _n = id;
        if let Some(n) = self.pin_group_aliases.get(_n) {
            _n = n;
        }
        if let Some(_pin_group) = self.pin_groups.get(_n) {
            Option::Some(_pin_group)
        } else {
            Option::None
        }
    }

    pub fn contains_pin_group(&mut self, id_or_alias: &str) -> bool {
        return self.pin_group(id_or_alias).is_some();
    }

    pub fn number_of_pin_groups(&mut self) -> usize {
        return self.pin_groups.len();
    }

    /// Given a pin id, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains_pin(&mut self, group_id: &str, pin_id: &str) -> bool {
        // Since we also need to check each pin's aliases, just brute force search it through the pins/aliases.
        return self.index_of(group_id, pin_id).is_some();
      }
  
      /// Given a pin or alias id, finds either its id or alias in the group.
      pub fn index_of(&mut self, group_id: &str, pin_id: &str) -> Option<usize> {
        let grp = self.get_pin_group(group_id).unwrap().pin_ids.clone();
        for (i, n) in grp.iter().enumerate() {
          let p = self.pin(n).unwrap();
          if pin_id == &(p.id) || p.aliases.contains(&pin_id.to_string()) {
            return Option::Some(i);
          }
        }
        Option::None
      }
    
      pub fn get_pin_group_data(&mut self, group_id: &str) -> u32 {
        let mut data = 0;
        let pin_ids = self.get_pin_group(group_id).unwrap().pin_ids.clone();
        for n in pin_ids.iter().rev() {
          let p = self.pin(n).unwrap();
          data = (data << 1) + p.data;
        }
        data as u32
      }
  
    pub fn verify_pin_group_data(&mut self, group_id: &str, data: u32) -> Result<(), Error> {
      let two: u32 = 2;
      let grp = self.get_pin_group(group_id).unwrap();
      if data > (two.pow(grp.len() as u32) - 1) {
        Err(Error::new(&format!("Data {} does not fit in Pin collection of size {} - Cannot set data!", data, grp.len())))
      } else {
        Ok(())
      }
    }

    pub fn set_pin_group_data(&mut self, group_id: &str, data: u32) -> Result<(), Error> {
      self.verify_pin_group_data(group_id, data)?;

      let mut d = data;
      let pin_ids = self.get_pin_group(group_id).unwrap().pin_ids.clone();
      for n in pin_ids.iter() {
        let p = self.pin(n).unwrap();
        p.set_data((d & 0x1) as u8)?;
        d = d >> 1;
      }
      Ok(())
    }

    /// Returns the pin actions as a string.
    /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the 
    ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
    pub fn get_pin_actions_for_group(&mut self, group_id: &str) -> String {
      let pin_ids = self.get_pin_group(group_id).unwrap().pin_ids.clone();
      let mut s = String::from("");
      for n in pin_ids.iter() {
        let p = self.pin(n).unwrap();
        s += &(p.action.as_char()).to_string();
      }
      s
    }

    pub fn set_pin_actions_for_group(&mut self, group_id: &str, action: PinActions, _mask: Option<u32>, data: Option<u32>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_group_data(group_id, d)?;
        }
        let grp = self.get_pin_group(group_id).unwrap();
        let pin_ids = grp.pin_ids.clone();
        for (_i, n) in pin_ids.iter().enumerate() {
            let p = self.pin(n).unwrap();
            p.action = action;
        }
        Ok(())
    }

    //pub fn with_mask

    pub fn drive_pin_group(&mut self, group_id: &str, mask: Option<u32>, data: Option<u32>) -> Result<(), Error> {
        return self.set_pin_actions_for_group(group_id, PinActions::Drive, mask, data);
    }

    pub fn verify_pin_group(&mut self, group_id: &str, mask: Option<u32>, data: Option<u32>) -> Result<(), Error> {
        return self.set_pin_actions_for_group(group_id, PinActions::Verify, mask, data);
    }

    pub fn capture_pin_group(&mut self, group_id: &str, mask: Option<u32>) -> Result<(), Error> {
        return self.set_pin_actions_for_group(group_id, PinActions::Capture, mask, Option::None);
    }

    pub fn highz_pin_group(&mut self, group_id: &str, mask: Option<u32>) -> Result<(), Error> {
        return self.set_pin_actions_for_group(group_id, PinActions::HighZ, mask, Option::None);
    }

    // Todo: Combine these with pin groups to cut down on the duplicate code

    pub fn get_pin_data(&mut self, ids: &Vec<String>) -> u32 {
        let mut data = 0;
        for n in ids.iter().rev() {
          let p = self.pin(n).unwrap();
          data = (data << 1) + p.data;
        }
        data as u32
    }

    pub fn set_pin_data(&mut self, ids: &Vec<String>, data: u32) -> Result<(), Error> {
        //self.verify_pin_group_data(group_id, data)?;

        let mut d = data;
        for n in ids.iter() {
          let p = self.pin(n).unwrap();
          p.set_data((d & 0x1) as u8)?;
          d = d >> 1;
        }
        Ok(())
    }

    pub fn get_pin_actions(&mut self, ids: &Vec<String>) -> Result<String, Error> {
        let mut s = String::from("");
        for n in ids.iter() {
          let p = self.pin(n).unwrap();
          s += &(p.action.as_char()).to_string();
        }
        Ok(s)
    }

    pub fn set_pin_actions(&mut self, ids: &Vec<String>, action: PinActions, data: Option<u32>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_data(ids, d)?;
        }
        for (_i, n) in ids.iter().enumerate() {
            let p = self.pin(n).unwrap();
            p.action = action;
        }
        Ok(())
    }

    pub fn drive_pins(&mut self, ids: &Vec<String>, data: Option<u32>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Drive, data)
    }

    pub fn verify_pins(&mut self, ids: &Vec<String>, data: Option<u32>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Verify, data)
    }

    pub fn capture_pins(&mut self, ids: &Vec<String>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Capture, Option::None)
    }

    pub fn highz_pins(&mut self, ids: &Vec<String>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::HighZ, Option::None)
    }

}