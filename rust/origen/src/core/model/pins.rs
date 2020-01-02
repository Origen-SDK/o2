pub mod pin;
pub mod pin_collection;
pub mod pin_group;
use crate::error::Error;
use std::convert::TryFrom;

extern crate regex;
use regex::Regex;

use super::Model;
use pin::{Pin, PinActions};
use pin_collection::PinCollection;
use pin_group::PinGroup;

#[derive(Debug, Copy, Clone)]
pub enum Endianness {
    LittleEndian,
    BigEndian,
}

impl Model {
    //** Functions for adding, aliasing, grouping, and collecting pins **

    pub fn add_pin(
        &mut self,
        name: &str,
        path: &str,
        width: Option<u32>,
        offset: Option<u32>,
        reset_data: Option<u32>,
        reset_action: Option<String>,
        endianness: Option<Endianness>,
    ) -> Result<&mut PinGroup, Error> {
        if self.get_pin_group(name).is_some() {
            return Err(Error::new(&format!(
                "Can not add pin {} because it conflicts with a current pin or alias name!",
                name
            )));
        }
        if !width.is_some() && offset.is_some() {
            return Err(Error::new(&format!(
                "Can not add pin {} with a given offset but no width option!",
                name
            )));
        }
        let mut names: Vec<String> = vec![];
        let (mut rdata, mut raction) = (Option::None, Option::None);

        if let Some(w) = width {
            if w < 1 {
                return Err(Error::new(&format!(
                    "Width cannot be less than 1! Received {}",
                    w
                )));
            }

            let (mut rdata_, mut raction_, mut raction_i, mut offset_) =
                (reset_data.unwrap_or(0), "".as_bytes(), w, 0);
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
                let _name = format!("{}{}", name, i);
                let p = Pin::new(String::from(&_name), String::from(path), rdata, raction);
                names.push(String::from(&p.name));
                self.pins.insert(
                    String::from(&_name),
                    PinGroup::new(
                        String::from(&_name),
                        String::from(path),
                        vec![String::from(&_name)],
                        endianness,
                    ),
                );
                self.physical_pins.insert(String::from(&_name), p);
            }
        } else {
            if let Some(d) = reset_data {
                // Single bit, so data can't be > 2.
                if d > 2 {
                    return Err(Error::new(&format!(
                        "Reset data of {} overflows available width (1)!",
                        d
                    )));
                }
                rdata = Option::Some(d);
            }
            if let Some(a) = reset_action {
                raction = Some(PinActions::from_str(&a)?);
            }
            let p = Pin::new(String::from(name), String::from(path), rdata, raction);
            names.push(String::from(&p.name));
            self.physical_pins.insert(String::from(name), p);
        }
        let grp = PinGroup::new(String::from(name), String::from(path), names, endianness);
        self.pins.insert(String::from(name), grp);
        Ok(self.pins.get_mut(name).unwrap())
    }

    pub fn add_pin_alias(&mut self, name: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if self.pins.contains_key(alias) {
            return Err(Error::new(&format!(
                "Could not alias pin {} to {}, as {} already exists!",
                name, alias, alias
            )));
        }

        let grp;
        let names;
        if let Some(p) = self.get_pin_group(name) {
            grp = PinGroup::new(
                String::from(alias),
                String::from(&p.path),
                p.pin_names.clone(),
                Option::Some(p.endianness),
            );
            names = p.pin_names.clone();
        } else {
            return Err(Error::new(&format!(
                "Could not alias pin {} to {}, as pin {} doesn't exists!",
                name, alias, name
            )));
        }
        for p in names.iter() {
            let pin = self._pin(&p).unwrap();
            pin.aliases.push(String::from(alias));
        }
        self.pins.insert(String::from(alias), grp);
        Ok(())
    }

    pub fn group_pins(
        &mut self,
        name: &str,
        path: &str,
        pins: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<&mut PinGroup, Error> {
        if self.get_pin_group(name).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias name!", name)));
        }

        let mut physical_names: Vec<String> = vec![];
        for (i, pin_name) in pins.iter().enumerate() {
            if let Some(p) = self.get_mut_physical_pin(pin_name) {
                if physical_names.contains(&p.name) {
                    return Err(Error::new(&format!("Can not group pins under {} because pin (or an alias of) {} has already been added to the group!", name, p.name)));
                } else {
                    p.groups.insert(String::from(name), i);
                }
            } else {
                return Err(Error::new(&format!(
                    "Can not group pins under {} because pin {} does not exist!",
                    name, pin_name
                )));
            }
            if let Some(p) = self.get_pin_group(pin_name) {
                physical_names.extend_from_slice(&p.pin_names);
            }
        }
        let grp = PinGroup::new(
            String::from(name),
            String::from(path),
            physical_names,
            endianness,
        );
        self.pins.insert(String::from(name), grp);
        Ok(self.pins.get_mut(name).unwrap())
    }

    // ** Functions for retrieving pins from names and aliases **

    /// Gets an immutable reference to an existing PinGroup, or Option::None, if not found..
    pub fn get_pin_group(&self, name: &str) -> Option<&PinGroup> {
        if let Some(pin) = self.pins.get(name) {
            Option::Some(pin)
        } else {
            Option::None
        }
    }

    /// Gets a mutable reference to an existing pin group, or Option::None, if not found.
    pub fn get_mut_pin_group(&mut self, name: &str) -> Option<&mut PinGroup> {
        if let Some(pin) = self.pins.get_mut(name) {
            Option::Some(pin)
        } else {
            Option::None
        }
    }

    /// Gets an immutable reference to an existing PinGroup, or an Error is the pin group isn't found.
    pub fn _get_pin_group(&self, name: &str) -> Result<&PinGroup, Error> {
        match self.get_pin_group(name) {
            Some(grp) => Ok(grp),
            None => Err(Error::new(&format!(
                "No pin group '{}' has been added!",
                name
            ))),
        }
    }

    /// Gets a mutable reference to an existing PinGroup, or an Error is the pin group isn't found.
    pub fn _get_mut_pin_group(&mut self, name: &str) -> Result<&mut PinGroup, Error> {
        match self.get_mut_pin_group(name) {
            Some(grp) => Ok(grp),
            None => Err(Error::new(&format!(
                "No pin group '{}' has been added!",
                name
            ))),
        }
    }

    pub fn _pin(&mut self, name: &str) -> Result<&mut Pin, Error> {
        match self.physical_pins.get_mut(name) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin {}! This signals either a bug in Origen or the backend model has been changed unexpectedly and this reference is stale.", name))),
        }
    }

    pub fn get_physical_pin(&self, name: &str) -> Option<&Pin> {
        if let Some(grp) = self.pins.get(name) {
            if let Some(physical_pin) = self.physical_pins.get(&grp.pin_names[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn get_mut_physical_pin(&mut self, name: &str) -> Option<&mut Pin> {
        if let Some(grp) = self.pins.get(name) {
            if let Some(physical_pin) = self.physical_pins.get_mut(&grp.pin_names[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn _get_physical_pin(&self, name: &str) -> Result<&Pin, Error> {
        match self.get_physical_pin(name) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin '{}'!", name))),
        }
    }

    pub fn _get_mut_physical_pin(&mut self, name: &str) -> Result<&mut Pin, Error> {
        match self.get_mut_physical_pin(name) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find phyiscal pin '{}'!", name))),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        return self.get_pin_group(name).is_some();
    }

    pub fn _contains(&self, name: &str) -> bool {
        return self.get_physical_pin(name).is_some();
    }

    /// Given a group/collection of pin names, verify:
    ///     * Each pin exist
    ///     * Each pin is unique (no duplicate pins) AND it points to a unique physical pin. That is, each pin is unique after resolving aliases.
    /// If all the above is met, we can group/collect these names.
    pub fn verify_names(&self, names: &Vec<String>) -> Result<Vec<String>, Error> {
        let mut physical_names: Vec<String> = vec![];
        for (_i, pin_name) in names.iter().enumerate() {
            if pin_name.starts_with("/") && pin_name.ends_with("/") {
                let mut regex_str = pin_name.clone();
                regex_str.pop();
                regex_str.remove(0);
                let regex = Regex::new(&regex_str).unwrap();

                let mut _pin_names: Vec<String> = vec![];
                for (name_str, grp) in self.pins.iter() {
                    if regex.is_match(name_str) {
                        for _name_str in grp.pin_names.iter() {
                            if physical_names.contains(_name_str) {
                                return Err(Error::new(&format!("Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", name_str, regex_str, _name_str)));
                            }
                        }
                        _pin_names.extend(grp.pin_names.clone())
                    }
                }
                _pin_names.sort();
                physical_names.extend(_pin_names);
            } else if let Some(p) = self.get_physical_pin(pin_name) {
                if physical_names.contains(&p.name) {
                    return Err(Error::new(&format!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", pin_name, p.name)));
                } else {
                    //physical_names.push(String::from(&p.name));
                }
                if let Some(p) = self.get_pin_group(pin_name) {
                    physical_names.extend_from_slice(&p.pin_names);
                }
            } else {
                return Err(Error::new(&format!(
                    "Can not collect pin '{}' because it does not exist!",
                    pin_name
                )));
            }
        }
        Ok(physical_names.clone())
    }

    pub fn collect(
        &mut self,
        model_id: usize,
        path: &str,
        names: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<PinCollection, Error> {
        let pnames = self.verify_names(&names)?;
        Ok(PinCollection::new(model_id, path, &pnames, endianness))
    }

    /// Given a pin name, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains(&mut self, name: &str, query_name: &str) -> Result<bool, Error> {
        let result = self.index_of(name, query_name)?.is_some();
        Ok(result)
    }

    /// Given a pin or alias name, finds either its name or alias in the group.
    pub fn index_of(&self, name: &str, query_name: &str) -> Result<Option<usize>, Error> {
        if !self.pins.contains_key(name) {
            // Pin group doesn't exists. Raise an error.
            return Err(Error::new(&format!(
                "Group {} does not exists! Cannot lookup index for {} in this group!",
                name, query_name
            )));
        }

        if let Some(p) = self.get_physical_pin(query_name) {
            if let Some(idx) = p.groups.get(name) {
                Ok(Option::Some(*idx))
            } else {
                // Group name wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query name doesn't exists. Raise an error.
            Err(Error::new(&format!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            )))
        }
    }

    pub fn pin_names_contain(
        &mut self,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<bool, Error> {
        let result = self.find_in_names(names, query_name)?.is_some();
        Ok(result)
    }

    pub fn find_in_names(
        &self,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<Option<usize>, Error> {
        if let Some(p) = self.get_physical_pin(query_name) {
            let idx = names
                .iter()
                .position(|name| p.name == *name || p.aliases.contains(name));
            if let Some(_idx) = idx {
                Ok(Option::Some(_idx))
            } else {
                // Group name wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query name doesn't exists. Raise an error.
            Err(Error::new(&format!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            )))
        }
    }

    pub fn data_fits_in_pins(&mut self, pins: &Vec<String>, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(pins.len() as u32) - 1) {
            Err(Error::new(&format!(
                "Data {} does not fit in Pin collection of size {} - Cannot set data!",
                data,
                pins.len()
            )))
        } else {
            Ok(())
        }
    }

    pub fn verify_data_fits(&mut self, width: u32, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(width) - 1) {
            Err(Error::new(&format!(
                "Data {} does not fit in pins with width of {}!",
                data, width
            )))
        } else {
            Ok(())
        }
    }

    pub fn verify_action_string_fits(&self, width: u32, action_string: &[u8]) -> Result<(), Error> {
        if action_string.len() != (width as usize) {
            Err(Error::new(&format!(
                "Action string of length {} must match width {}!",
                action_string.len(),
                width
            )))
        } else {
            Ok(())
        }
    }

    pub fn get_pin_data(&self, names: &Vec<String>) -> u32 {
        let mut data = 0;
        for n in names.iter().rev() {
            let p = self.get_physical_pin(n).unwrap();
            data = (data << 1) + p.data;
        }
        data as u32
    }

    pub fn get_pin_reset_data(&self, names: &Vec<String>) -> u32 {
        let mut rdata = 0;
        for n in names.iter().rev() {
            let p = self.get_physical_pin(n).unwrap();
            rdata = (rdata << 1) + p.reset_data.unwrap_or(0);
        }
        rdata as u32
    }

    pub fn reset_pin_names(&mut self, names: &Vec<String>) -> Result<(), Error> {
        for n in names.iter() {
            let p = self.get_mut_physical_pin(n).unwrap();
            p.reset();
        }
        Ok(())
    }

    pub fn set_pin_data(
        &mut self,
        names: &Vec<String>,
        data: u32,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        self.data_fits_in_pins(names, data)?;

        let mut d = data;
        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for n in names.iter() {
            let p = self._pin(n).unwrap();
            p.set_data(((d & 0x1) & (m & 0x1)) as u8)?;
            d = d >> 1;
            m = m >> 1;
        }
        Ok(())
    }

    pub fn get_pin_actions(&mut self, names: &Vec<String>) -> Result<String, Error> {
        let mut s = String::from("");
        for n in names.iter() {
            let p = self._pin(n).unwrap();
            s += &(p.action.as_char()).to_string();
        }
        Ok(s)
    }

    pub fn get_pin_reset_actions(&mut self, names: &Vec<String>) -> Result<String, Error> {
        let mut s = String::from("");
        for n in names.iter() {
            let p = self._pin(n).unwrap();
            s += &(p.reset_action.unwrap_or(PinActions::HighZ).as_char()).to_string();
        }
        Ok(s)
    }

    pub fn set_pin_actions(
        &mut self,
        names: &Vec<String>,
        action: PinActions,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_data(names, d, mask)?;
        }

        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for (_i, n) in names.iter().rev().enumerate() {
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

    pub fn resolve_pin_names(&mut self, names: &Vec<String>) -> Result<Vec<String>, Error> {
        let mut physical_names: Vec<String> = vec![];
        for (_i, n) in names.iter().enumerate() {
            let p = self._pin(n).unwrap();
            physical_names.push(p.name.clone());
        }
        Ok(physical_names)
    }

    pub fn drive_pins(
        &mut self,
        names: &Vec<String>,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        self.set_pin_actions(names, PinActions::Drive, data, mask)
    }

    pub fn verify_pins(
        &mut self,
        names: &Vec<String>,
        data: Option<u32>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        self.set_pin_actions(names, PinActions::Verify, data, mask)
    }

    pub fn capture_pins(&mut self, names: &Vec<String>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(names, PinActions::Capture, Option::None, mask)
    }

    pub fn highz_pins(&mut self, names: &Vec<String>, mask: Option<usize>) -> Result<(), Error> {
        self.set_pin_actions(names, PinActions::HighZ, Option::None, mask)
    }
}
