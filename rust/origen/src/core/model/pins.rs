pub mod pin;
pub mod pin_group;
pub mod pin_collection;
use crate::error::Error;
use std::convert::TryFrom;

extern crate regex;
use regex::Regex;

use pin_collection::{PinCollection};
use pin::{Pin, PinActions};
use pin_group::PinGroup;
use super::Model;

#[derive(Debug, Copy, Clone)]
pub enum Endianness {
  LittleEndian, BigEndian
}

impl Model {

    //** Functions for adding, aliasing, grouping, and collecting pins **

    pub fn add_pin(&mut self, id: &str, path: &str, width: Option<u32>, offset: Option<u32>, reset_data: Option<u32>, reset_action: Option<String>, endianness: Option<Endianness>) -> Result<&mut PinGroup, Error> {
        let n = id;
        if self.get_pin_group(id).is_some() {
            return Err(Error::new(&format!("Can not add pin {} because it conflicts with a current pin or alias id!", id)))
        }
        if !width.is_some() && offset.is_some() {
            return Err(Error::new(&format!("Can not add pin {} with a given offset but no width option!", id)))
        }
        let mut ids: Vec<String> = vec!();
        let (mut rdata, mut raction) = (Option::None, Option::None);

        if let Some(w) = width {
            if w < 1 {
                return Err(Error::new(&format!("Width cannot be less than 1! Received {}", w)));
            }
            
            let (mut rdata_, mut raction_, mut raction_i, mut offset_) = (reset_data.unwrap_or(0), "".as_bytes(), w, 0);
            self.verify_data_fits(w, rdata_)?;
            let mut temp = String::from("");
            if let Some(o) = offset {
                offset_ = o;
            }
            if let Some(r) = reset_action {
                temp = r;
                raction_ = temp.as_bytes();
                self.verify_action_string_fits(w, raction_)?;
            }
            for i in offset_..(offset_ + w) {
                if reset_data.is_some() {
                    // Set the individual pin data one by one, shifting the available data by one each time.
                    rdata = Some(rdata_ & 0x1);
                    rdata_ >>= 1;
                }
                if temp != "" {
                    // Same with the pin actions, except we'll be 'shifting' by a character each time.
                    // Note: we're assuming an action string inline with how we'd read data, so we're actually reading and shifting out of the
                    //  end of the string.
                    // Note: a single character can be given to apply all the same action to all pins in the width.
                    if raction_.len() > 1 {
                        raction = Some(PinActions::try_from(raction_[(raction_i - 1) as usize])?);
                        raction_i -= 1;
                    } else {
                        raction = Some(PinActions::try_from(raction_[0])?);
                    }
                }
                let id = format!("{}{}", n, i);
                let p = Pin::new(String::from(&id), String::from(path), rdata, raction);
                ids.push(String::from(&p.id));
                self.pins.insert(String::from(&id), PinGroup::new(String::from(&id), String::from(path), vec!(String::from(&id)), endianness));
                self.physical_pins.insert(String::from(&id), p);
            }
        } else {
            if let Some(d) = reset_data {
                // Single bit, so data can't be > 2.
                if d > 2 {
                    return Err(Error::new(&format!("Reset data of {} overflows available width (1)!", d)));
                }
                rdata = Option::Some(d);
            }
            if let Some(a) = reset_action {
                raction = Some(PinActions::from_str(&a)?);
            }
            let p = Pin::new(String::from(n), String::from(path), rdata, raction);
            ids.push(String::from(&p.id));
            self.physical_pins.insert(String::from(n), p);
        }
        let grp = PinGroup::new(String::from(n), String::from(path), ids, endianness);
        self.pins.insert(String::from(n), grp);
        Ok(self.pins.get_mut(n).unwrap())
    }

    pub fn add_pin_alias(&mut self, id: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if self.pins.contains_key(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as {} already exists!", id, alias, alias)))
        }

        let grp;
        let ids;
        if let Some(p) = self.get_pin_group(id) {
            grp = PinGroup::new (
                String::from(alias),
                String::from(&p.path),
                //vec!(String::from(&p.id)),
                p.pin_ids.clone(),
                Option::Some(p.endianness),
            );
            ids = p.pin_ids.clone();
        } else {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} doesn't exists!", id, alias, id)))
        }
        for p in ids.iter() {
            let pin = self._pin(&p).unwrap();
            pin.aliases.push(String::from(alias));
        }
        self.pins.insert(String::from(alias), grp);
        Ok(())
    }

    pub fn group_pins(&mut self, id: &str, path: &str, pins: Vec<String>, endianness: Option<Endianness>) -> Result<&mut PinGroup, Error> {
        let n = id;
        if self.get_pin_group(id).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias id!", id)))
        }

        let mut physical_ids: Vec<String> = vec!();
        for (i, pin_id) in pins.iter().enumerate() {
            if let Some(p) = self.get_mut_physical_pin(pin_id) {
                if physical_ids.contains(&p.id) {
                    return Err(Error::new(&format!("Can not group pins under {} because pin (or an alias of) {} has already been added to the group!", id, p.id)));
                } else {
                    p.groups.insert(String::from(n), i);
                }
            } else {
                return Err(Error::new(&format!("Can not group pins under {} because pin {} does not exist!", id, pin_id)));
            }
            if let Some(p) = self.get_pin_group(pin_id) {
                physical_ids.extend_from_slice(&p.pin_ids);
            }
        }
        let grp = PinGroup::new(String::from(n), String::from(path), physical_ids, endianness);
        self.pins.insert(String::from(n), grp);
        Ok(self.pins.get_mut(n).unwrap())
    }

    // ** Functions for retrieving pins from ids and aliases **

    /// Gets an immutable reference to an existing PinGroup, or Option::None, if not found..
    pub fn get_pin_group(&self, id: &str) -> Option<&PinGroup> {
        if let Some(pin) = self.pins.get(id) {
            Option::Some(pin)
        } else {
            Option::None
        }
    }

    /// Gets a mutable reference to an existing pin group, or Option::None, if not found.
    pub fn get_mut_pin_group(&mut self, id: &str) -> Option<&mut PinGroup> {
        if let Some(pin) = self.pins.get_mut(id) {
            Option::Some(pin)
        } else {
            Option::None
        }
    }

    /// Gets an immutable reference to an existing PinGroup, or an Error is the pin group isn't found.
    pub fn _get_pin_group(&self, id: &str) -> Result<&PinGroup, Error> {
        match self.get_pin_group(id) {
            Some(grp) => Ok(grp),
            None => Err(Error::new(&format!("No pin group '{}' has been added!", id))),
        }
    }

    /// Gets a mutable reference to an existing PinGroup, or an Error is the pin group isn't found.
    pub fn _get_mut_pin_group(&mut self, id: &str) -> Result<&mut PinGroup, Error> {
        match self.get_mut_pin_group(id) {
            Some(grp) => Ok(grp),
            None => Err(Error::new(&format!("No pin group '{}' has been added!", id))),
        }
    }

    pub fn _pin(&mut self, id: &str) -> Result<&mut Pin, Error> {
        match self.physical_pins.get_mut(id) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin {}! This signals either a bug in Origen or the backend model has been changed unexpectedly and this reference is stale.", id))),
        }
    }

    pub fn get_physical_pin(&self, id: &str) -> Option<&Pin> {
        if let Some(grp) = self.pins.get(id) {
            if let Some(physical_pin) = self.physical_pins.get(&grp.pin_ids[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn get_mut_physical_pin(&mut self, id: &str) -> Option<&mut Pin> {
        if let Some(grp) = self.pins.get(id) {
            if let Some(physical_pin) = self.physical_pins.get_mut(&grp.pin_ids[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn _get_physical_pin(&self, id: &str) -> Result<&Pin, Error> {
        match self.get_physical_pin(id) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin '{}'!", id))),
        }
    }

    pub fn _get_mut_physical_pin(&mut self, id: &str) -> Result<&mut Pin, Error> {
        match self.get_mut_physical_pin(id) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin '{}'!", id))),
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        return self.get_pin_group(id).is_some();
    }

    pub fn _contains(&self, id: &str) -> bool {
        return self.get_physical_pin(id).is_some();
    }

    /// Given a group/collection of pin IDs, verify:
    ///     * Each pin exist
    ///     * Each pin is unique (no duplicate pins) AND it points to a unique physical pin. That is, each pin is unique after resolving aliases.
    /// If all the above is met, we can group/collect these IDs.
    pub fn verify_ids(&self, ids: &Vec<String>) -> Result<Vec<String>, Error> {
        let mut physical_ids: Vec<String> = vec!();
        for (_i, pin_id) in ids.iter().enumerate() {
            if pin_id.starts_with("/") && pin_id.ends_with("/") {
                let mut regex_str = pin_id.clone();
                regex_str.pop();
                regex_str.remove(0);
                let regex = Regex::new(&regex_str).unwrap();

                let mut _pin_ids: Vec<String> = vec!();
                for (id_str, grp) in self.pins.iter() {
                    if regex.is_match(id_str) {
                        for _id_str in grp.pin_ids.iter() {
                            if physical_ids.contains(_id_str) {
                                return Err(Error::new(&format!("Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", id_str, regex_str, _id_str)));
                            }
                        }
                        _pin_ids.extend(grp.pin_ids.clone())
                    }
                }
                _pin_ids.sort();
                physical_ids.extend(_pin_ids);
            } else if let Some(p) = self.get_physical_pin(pin_id) {
                if physical_ids.contains(&p.id) {
                    return Err(Error::new(&format!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", pin_id, p.id)));
                } else {
                    //physical_ids.push(String::from(&p.id));
                }
                if let Some(p) = self.get_pin_group(pin_id) {
                    physical_ids.extend_from_slice(&p.pin_ids);
                }
            } else {
                return Err(Error::new(&format!("Can not collect pin '{}' because it does not exist!", pin_id)));
            }
        }
        Ok(physical_ids.clone())
    }

    pub fn collect(&mut self, model_id: usize, path: &str, ids: Vec<String>, endianness: Option<Endianness>) -> Result<PinCollection, Error> {
        let pids = self.verify_ids(&ids)?;
        Ok(PinCollection::new(model_id, path, &pids, endianness))
    }

    /// Given a pin id, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains(&mut self, id: &str, query_id: &str) -> Result<bool, Error> {
        let result = self.index_of(id, query_id)?.is_some();
        Ok(result)
    }
  
    /// Given a pin or alias id, finds either its id or alias in the group.
    pub fn index_of(&self, id: &str, query_id: &str) -> Result<Option<usize>, Error> {
        if !self.pins.contains_key(id) {
            // Pin group doesn't exists. Raise an error.
            return Err(Error::new(&format!("Group {} does not exists! Cannot lookup index for {} in this group!", id, query_id)));
        }

        if let Some(p) = self.get_physical_pin(query_id) {
            if let Some(idx) = p.groups.get(id) {
                Ok(Option::Some(*idx))
            } else {
                // Group ID wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query ID doesn't exists. Raise an error.
            Err(Error::new(&format!("The query ID {} does not exists! Cannot check this query's groups!", query_id)))
        }
    }

    pub fn pin_ids_contain(&mut self, ids: &Vec<String>, query_id: &str) -> Result<bool, Error> {
        let result = self.find_in_ids(ids, query_id)?.is_some();
        Ok(result)
    }

    pub fn find_in_ids(&self, ids: &Vec<String>, query_id: &str) -> Result<Option<usize>, Error> {
        if let Some(p) = self.get_physical_pin(query_id) {
            let idx = ids.iter().position( |id| p.id == *id || p.aliases.contains(id));
            if let Some(_idx) = idx {
                Ok(Option::Some(_idx))
            } else {
                // Group ID wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query ID doesn't exists. Raise an error.
            Err(Error::new(&format!("The query ID {} does not exists! Cannot check this query's groups!", query_id)))
        }
    }

    pub fn data_fits_in_pins(&mut self, pins: &Vec<String>, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(pins.len() as u32) - 1) {
            Err(Error::new(&format!("Data {} does not fit in Pin collection of size {} - Cannot set data!", data, pins.len())))
        } else {
            Ok(())
        }
    }

    pub fn verify_data_fits(&mut self, width: u32, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(width) - 1) {
            Err(Error::new(&format!("Data {} does not fit in pins with width of {}!", data, width)))
        } else {
            Ok(())
        }
    }

    pub fn verify_action_string_fits(&self, width: u32, action_string: &[u8]) -> Result<(), Error> {
        if action_string.len() != (width as usize) {
            Err(Error::new(&format!("Action string of length {} must match width {}!", action_string.len(), width)))
        } else {
            Ok(())
        }
    }

    pub fn get_pin_data(&self, ids: &Vec<String>) -> u32 {
        let mut data = 0;
        for n in ids.iter().rev() {
          let p = self.get_physical_pin(n).unwrap();
          data = (data << 1) + p.data;
        }
        data as u32
    }

    pub fn get_pin_reset_data(&self, ids: &Vec<String>) -> u32 {
        let mut rdata = 0;
        for n in ids.iter().rev() {
          let p = self.get_physical_pin(n).unwrap();
          rdata = (rdata << 1) + p.reset_data.unwrap_or(0);
        }
        rdata as u32
    }


    pub fn reset_pin_ids(&mut self, ids: &Vec<String>) -> Result<(), Error> {
        for n in ids.iter() {
          let p = self.get_mut_physical_pin(n).unwrap();
          p.reset();
        }
        Ok(())
    }

    pub fn set_pin_data(&mut self, ids: &Vec<String>, data: u32, mask: Option<usize>) -> Result<(), Error> {
        self.data_fits_in_pins(ids, data)?;

        let mut d = data;
        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for n in ids.iter() {
          let p = self._pin(n).unwrap();
          p.set_data(((d & 0x1) & (m & 0x1)) as u8)?;
          d = d >> 1;
          m = m >> 1;
        }
        Ok(())
    }

    pub fn get_pin_actions(&mut self, ids: &Vec<String>) -> Result<String, Error> {
        let mut s = String::from("");
        for n in ids.iter() {
          let p = self._pin(n).unwrap();
          s += &(p.action.as_char()).to_string();
        }
        Ok(s)
    }

    pub fn get_pin_reset_actions(&mut self, ids: &Vec<String>) -> Result<String, Error> {
        let mut s = String::from("");
        for n in ids.iter() {
          let p = self._pin(n).unwrap();
          s += &(p.reset_action.unwrap_or(PinActions::HighZ).as_char()).to_string();
        }
        Ok(s)
    }

    pub fn set_pin_actions(&mut self, ids: &Vec<String>, action: PinActions, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_data(ids, d, mask)?;
        }

        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for (_i, n) in ids.iter().rev().enumerate() {
            let p = self._pin(n).unwrap();

            if m & 0x1 == 1 {
                p.action = action;
            } else {
                p.action = PinActions::HighZ;
            }
            m >>= 1;
        }
        Ok(())
    }

    pub fn resolve_pin_ids(&mut self, ids: &Vec<String>) -> Result<Vec<String>, Error> {
        let mut physical_ids: Vec<String> = vec!();
        for (_i, n) in ids.iter().enumerate() {
            let p = self._pin(n).unwrap();
            physical_ids.push(p.id.clone());
        }
        Ok(physical_ids)
    }

    pub fn drive_pins(&mut self, ids: &Vec<String>, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Drive, data, mask)
    }

    pub fn verify_pins(&mut self, ids: &Vec<String>, data: Option<u32>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Verify, data, mask)
    }

    pub fn capture_pins(&mut self, ids: &Vec<String>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::Capture, Option::None, mask)
    }

    pub fn highz_pins(&mut self, ids: &Vec<String>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(ids, PinActions::HighZ, Option::None, mask)
    }
}
