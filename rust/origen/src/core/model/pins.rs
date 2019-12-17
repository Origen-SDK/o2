pub mod pin;
pub mod pin_group;
pub mod pin_collection;
use crate::error::Error;

use std::collections::HashMap;
//use pin_collection::{PinCollection};
use pin::Pin;
use pin_group::PinGroup;
use super::Model;

impl Model {
    pub fn add_pin(&mut self, name: &str, path: &str) -> Result<&mut Pin, Error> {
        let n = name;
        if self.pin(name).is_some() {
            return Err(Error::new(&format!("Can not add pin {} because it conflicts with a current pin or alias name!", name)))
        }
        let p = Pin::new(String::from(n), String::from(path));
        self.pins.insert(String::from(n), p);
        Ok(self.pins.get_mut(n).unwrap())
    }

    /// Returns the phyiscal pin, or None, if the pin doesn't exist.
    /// Implementation note: based on anecdotal evidence, physical pins generally have hardware-oriented pins, e.g.: PTA0, GPIO_A, etc.
    ///   The app will alias those to friendlier names like swd_io, swdclk, which patterns and drivers will use.
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

    //pub fn names_for(&mut self, pin:)
    
    /// Returns a HashMap with all pin names and, in the event of an alias,the pin name its aliased to.
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

    pub fn has_pin(&mut self, name_or_alias: &str) -> bool {
        return self.pin(name_or_alias).is_some();
    }

    pub fn number_of_pins(&mut self) -> usize {
        return self.pins.len();
    }

    pub fn add_pin_alias(&mut self, name: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if !self.pins.contains_key(name) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} doesn't exists!", name, alias, name)))
        }
        if self.pins.contains_key(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as alias {} already exists as a pin!", name, alias, alias)))
        }

        // Check if the alias already exists. If so, raise an exception, otherwise add the alias.
        if let Some(_name) = self.pin_aliases.get(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} is already aliased to {}", name, alias, _name, alias)))
        } else {
            self.pin_aliases.insert(String::from(alias), String::from(name));
            Ok(())
        }
    }

    pub fn group_pins(&mut self, name: &str, path: &str, pins: Vec<String>) -> Result<&mut PinGroup, Error> {
        let n = name;
        if self.pin_group(name).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias name!", name)))
        }
        let p = PinGroup::new(String::from(n), String::from(path), pins);
        self.pin_groups.insert(String::from(n), p);
        Ok(self.pin_groups.get_mut(n).unwrap())
    }

    pub fn pin_group(&mut self, name: &str) -> Option<&mut PinGroup> {
        let mut _n = name;
        if let Some(n) = self.pin_group_aliases.get(_n) {
            _n = n;
        }
        if let Some(_pin_group) = self.pin_groups.get_mut(_n) {
            Option::Some(_pin_group)
        } else {
            Option::None
        }
    }

    pub fn get_pin_group(&self, name: &str) -> Option<&PinGroup> {
        let mut _n = name;
        if let Some(n) = self.pin_group_aliases.get(_n) {
            _n = n;
        }
        if let Some(_pin_group) = self.pin_groups.get(_n) {
            Option::Some(_pin_group)
        } else {
            Option::None
        }
    }

    pub fn contains_pin_group(&mut self, name_or_alias: &str) -> bool {
        return self.pin_group(name_or_alias).is_some();
    }

    pub fn number_of_pin_groups(&mut self) -> usize {
        return self.pin_groups.len();
    }

    /// Given a pin name, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains_pin(&mut self, group_name: &str, pin_name: &str) -> bool {
        // Since we also need to check each pin's aliases, just brute force search it through the pins/aliases.
        return self.index_of(group_name, pin_name).is_some();
      }
  
      /// Given a pin or alias name, finds either its name or alias in the group.
      pub fn index_of(&mut self, group_name: &str, pin_name: &str) -> Option<usize> {
        let grp = self.get_pin_group(group_name).unwrap().pin_names.clone();
        for (i, n) in grp.iter().enumerate() {
          let p = self.pin(n).unwrap();
          if pin_name == &(p.name) || p.aliases.contains(&pin_name.to_string()) {
            return Option::Some(i);
          }
        }
        Option::None
      }
    
      pub fn get_pin_group_data(&mut self, group_name: &str) -> u32 {
        let mut data = 0;
        let pin_names = self.get_pin_group(group_name).unwrap().pin_names.clone();
        for n in pin_names.iter().rev() {
          let p = self.pin(n).unwrap();
          data = (data << 1) + p.data;
        }
        data as u32
      }
  
    pub fn verify_pin_group_data(&mut self, group_name: &str, data: u32) -> Result<(), Error> {
      let two: u32 = 2;
      let grp = self.get_pin_group(group_name).unwrap();
      if data > (two.pow(grp.len() as u32) - 1) {
        Err(Error::new(&format!("Data {} does not fit in Pin collection of size {} - Cannot set data!", data, grp.len())))
      } else {
        Ok(())
      }
    }

    pub fn set_pin_group_data(&mut self, group_name: &str, data: u32) -> Result<(), Error> {
      self.verify_pin_group_data(group_name, data)?;

      let mut d = data;
      let pin_names = self.get_pin_group(group_name).unwrap().pin_names.clone();
      for n in pin_names.iter() {
        let p = self.pin(n).unwrap();
        p.set_data((d & 0x1) as u8)?;
        d = d >> 1;
      }
      Ok(())
    }

    /// Returns the pin actions as a string.
    /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the 
    ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
    pub fn get_pin_actions_for_group(&mut self, group_name: &str) -> String {
      let pin_names = self.get_pin_group(group_name).unwrap().pin_names.clone();
      let mut s = String::from("");
      for n in pin_names.iter() {
        let p = self.pin(n).unwrap();
        s += &(p.action.as_char()).to_string();
      }
      s
    }

    // pub fn drive(&self, pin_container: &mut PinContainer, data: Option<usize>) -> Result<(), Error> {
    //   if let Some(d) = data {
    //     self.set_data(d as u32)?;
    //   }
    //   for n in self.pin_names.iter() {
    //     let p = pin_container.pin(n).unwrap();
    //     p.action = PinActions::Drive;
    //   }
    //   Ok(())
    // }
    // // ...
}