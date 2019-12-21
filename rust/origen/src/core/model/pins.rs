pub mod pin;
pub mod pin_group;
pub mod pin_collection;
use crate::error::Error;

use pin_collection::{PinCollection, Endianness};
use pin::{Pin, PinActions};
use pin_group::PinGroup;
use super::Model;

impl Model {

    //** Functions for adding, aliasing, grouping, and collecting pins **

    pub fn add_pin(&mut self, id: &str, path: &str) -> Result<&mut PinGroup, Error> {
        let n = id;
        if self.pin(id).is_some() {
            return Err(Error::new(&format!("Can not add pin {} because it conflicts with a current pin or alias id!", id)))
        }
        let p = Pin::new(String::from(n), String::from(path));
        let grp = PinGroup::new(String::from(n), String::from(path), vec!(String::from(n)));
        self.pins.insert(String::from(n), grp);
        self.physical_pins.insert(String::from(n), p);
        Ok(self.pins.get_mut(n).unwrap())
    }

    pub fn add_pin_alias(&mut self, id: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if self.pins.contains_key(alias) {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as {} already exists!", id, alias, alias)))
        }

        let grp;
        if let Some(p) = self.mut_physical_pin(id) {
            grp = PinGroup::new (
                String::from(alias),
                String::from(&p.path),
                vec!(String::from(&p.id)),
            );
            p.aliases.push(String::from(alias));
        } else {
            return Err(Error::new(&format!("Could not alias pin {} to {}, as pin {} doesn't exists!", id, alias, id)))
        }
        self.pins.insert(String::from(alias), grp);
        Ok(())
    }

    pub fn group_pins(&mut self, id: &str, path: &str, pins: Vec<String>) -> Result<&mut PinGroup, Error> {
        let n = id;
        if self.pin(id).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias id!", id)))
        }
        let grp = PinGroup::new(String::from(n), String::from(path), pins);
        let mut seen_physical_ids: Vec<String> = vec!();
        for (i, pin_id) in grp.pin_ids.iter().enumerate() {
            if let Some(p) = self.mut_physical_pin(pin_id) {
                if seen_physical_ids.contains(&p.id) {
                    return Err(Error::new(&format!("Can not group pins under {} because pin (or an alias of) {} has already been added to the group!", id, p.id)));
                } else {
                    seen_physical_ids.push(String::from(&p.id));
                    p.groups.insert(String::from(n), i);
                }
            } else {
                return Err(Error::new(&format!("Can not group pins under {} because pin {} does not exist!", id, pin_id)));
            }
        }
        self.pins.insert(String::from(n), grp);
        Ok(self.pins.get_mut(n).unwrap())
    }

    // ** Functions for retrieving pins from ids and aliases **

    pub fn pin(&mut self, id: &str) -> Option<&mut PinGroup> {
        if let Some(pin) = self.pins.get_mut(id) {
            Option::Some(pin)
        } else {
            Option::None
        }
    }

    pub fn _pin(&mut self, id: &str) -> Result<&mut Pin, Error> {
        match self.physical_pins.get_mut(id) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin {}! This signals either a bug in Origen or the backend model has been changed unexpectedly and this reference is stale.", id))),
        }
    }

    pub fn physical_pin(&self, id: &str) -> Option<&Pin> {
        if let Some(grp) = self.pins.get(id) {
            if let Some(physical_pin) = self.physical_pins.get(&grp.pin_ids[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn mut_physical_pin(&mut self, id: &str) -> Option<&mut Pin> {
        if let Some(grp) = self.pins.get(id) {
            if let Some(physical_pin) = self.physical_pins.get_mut(&grp.pin_ids[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn contains(&mut self, id: &str) -> bool {
        return self.pin(id).is_some();
    }

    pub fn _contains(&mut self, id: &str) -> bool {
        return self.physical_pin(id).is_some();
    }

    pub fn number_of_physical_pins(&mut self) -> usize {
        return self.physical_pins.len();
    }

    pub fn number_of_ids(&mut self) -> usize {
        return self.pins.len();
    }

    /// Given a group/collection of pin IDs, verify:
    ///     * Each pin exist
    ///     * Each pin is unique (no duplicate pins) AND it points to a unique physical pin. That is, each pin is unique after resolving aliases.
    /// If all the above is met, we can group/collect these IDs.
    pub fn verify_ids(&self, ids: &Vec<String>) -> Result<(), Error> {
        let mut seen_physical_ids: Vec<String> = vec!();
        for (_i, pin_id) in ids.iter().enumerate() {
            if let Some(p) = self.physical_pin(pin_id) {
                if seen_physical_ids.contains(&p.id) {
                    return Err(Error::new(&format!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", pin_id, p.id)));
                } else {
                    seen_physical_ids.push(String::from(&p.id));
                }
            } else {
                return Err(Error::new(&format!("Can not collect pin '{}' because it does not exist!", pin_id)));
            }
        }
        Ok(())
    }

    pub fn collect(&mut self, path: &str, ids: Vec<String>) -> Result<PinCollection, Error> {
        self.verify_ids(&ids)?;
        Ok(PinCollection {
            path: String::from(path),
            ids: ids,
            endianness: Endianness::LittleEndian,
        })
    }

    /// Given a pin id, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains(&mut self, id: &str, query_id: &str) -> Result<bool, Error> {
        let result = self.index_of(id, query_id)?.is_some();
        Ok(result)
    }
  
    /// Given a pin or alias id, finds either its id or alias in the group.
    pub fn index_of(&mut self, id: &str, query_id: &str) -> Result<Option<usize>, Error> {
        if !self.pins.contains_key(id) {
            // Pin group doesn't exists. Raise an error.
            return Err(Error::new(&format!("Group {} does not exists! Cannot lookup index for {} in this group!", id, query_id)));
        }

        if let Some(p) = self.physical_pin(query_id) {
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

    pub fn find_in_ids(&mut self, ids: &Vec<String>, query_id: &str) -> Result<Option<usize>, Error> {
        if let Some(p) = self.physical_pin(query_id) {
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

    pub fn get_pin_group_data(&mut self, id: &str) -> u32 {
        let pin_ids = self.pin(id).unwrap().pin_ids.clone();
        self.get_pin_data(&pin_ids)
    }

    pub fn set_pin_group_data(&mut self, id: &str, data: u32) -> Result<(), Error> {
        let pin_ids = self.pin(id).unwrap().pin_ids.clone();
        self.set_pin_data(&pin_ids, data)
    }

    pub fn resolve_pin_group_ids(&mut self, id: &str) -> Result<Vec<String>, Error> {
        let pin_ids = self.pin(id).unwrap().pin_ids.clone();
        self.resolve_pin_ids(&pin_ids)
    }

    /// Returns the pin actions as a string.
    /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the 
    ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
    pub fn get_pin_actions_for_group(&mut self, id: &str) -> Result<String, Error> {
      let pin_ids = self.pin(id).unwrap().pin_ids.clone();
      self.get_pin_actions(&pin_ids)
    }

    pub fn set_pin_actions_for_group(&mut self, id: &str, action: PinActions, _mask: Option<u32>, data: Option<u32>) -> Result<(), Error> {
        let pin_ids = self.pin(id).unwrap().pin_ids.clone();
        self.set_pin_actions(&pin_ids, action, data)
    }

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

    pub fn get_pin_data(&mut self, ids: &Vec<String>) -> u32 {
        let mut data = 0;
        for n in ids.iter().rev() {
          let p = self._pin(n).unwrap();
          data = (data << 1) + p.data;
        }
        data as u32
    }

    pub fn set_pin_data(&mut self, ids: &Vec<String>, data: u32) -> Result<(), Error> {
        self.data_fits_in_pins(ids, data)?;

        let mut d = data;
        for n in ids.iter() {
          let p = self._pin(n).unwrap();
          p.set_data((d & 0x1) as u8)?;
          d = d >> 1;
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

    pub fn set_pin_actions(&mut self, ids: &Vec<String>, action: PinActions, data: Option<u32>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_data(ids, d)?;
        }
        for (_i, n) in ids.iter().enumerate() {
            let p = self._pin(n).unwrap();
            p.action = action;
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

    // Assume the pin group is properly defined (that is, not pin duplicates and all pins exists. If the pin group exists, these should both be met)
    pub fn slice_pin_group(&mut self, id: &str, start_idx: usize, stop_idx: usize, step_size: usize) -> Result<PinCollection, Error> {
        if let Some(p) = self.pin(id) {
            let ids = &p.pin_ids;
            let mut sliced_ids: Vec<String> = vec!();

            for i in (start_idx..=stop_idx).step_by(step_size) {
                if i >= ids.len() {
                    return Err(Error::new(&format!("Index {} exceeds available pins in group {} (length: {})", i, id, ids.len())));
                }
                let p = ids[i].clone();
                sliced_ids.push(p);
            }
            Ok(PinCollection::new(&self.parent_path, &sliced_ids, Option::None))
        } else {
            Err(Error::new(&format!("Could not slice pin {} because it doesn't exists!", id)))
        }
    }
}
