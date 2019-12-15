pub mod pin;
pub mod pin_collection;
use crate::error::Error;

use std::collections::HashMap;
use pin_collection::{PinCollection};

#[derive(Debug)]
pub enum Types {
    Pin(pin::Pin), PinGroup(PinCollection)
}

#[derive(Debug)]
pub enum AliasTypes {
    PinAlias(String), PinGroupAlias(String)
}

/// Structure to contain the added pins.
#[derive(Debug)]
pub struct PinContainer {
    pub pins: HashMap<String, pin::Pin>,
    pub pin_aliases: HashMap<String, String>,
    pub pin_groups: HashMap<String, pin::PinGroup>,
    pub pin_group_aliases: HashMap<String, String>,

    // The aliases will be String that points to a the aliased pin/group's name (as a String)
    // e.g.: alias('test', 'alias') -> pin_aliases['test'] == 'alias'
    //pub pin_aliases: HashMap<String, String>,
    //pub group_aliases: <String, String>,
    //pub contents: HashMap<String, Types>,
    //pub aliases: HashMap<String, AliasTypes>,
}

impl PinContainer {
    pub fn new() -> PinContainer {
        PinContainer {
            pins: HashMap::new(),
            pin_aliases: HashMap::new(),
            pin_groups: HashMap::new(),
            pin_group_aliases: HashMap::new(),
        }
    }

    pub fn add_pin(&mut self, name: &str, path: &str) -> Result<&mut pin::Pin, Error> {
        let n = name;
        if self.pin(name).is_some() {
            return Err(Error::new(&format!("Can not add pin {} because it conflicts with a current pin or alias name!", name)))
        }
        let p = pin::Pin::new(String::from(n), String::from(path));
        self.pins.insert(String::from(n), p);
        Ok(self.pins.get_mut(n).unwrap())
    }

    /// Returns the phyiscal pin, or None, if the pin doesn't exist.
    /// Implementation note: based on anecdotal evidence, physical pins generally have hardware-oriented pins, e.g.: PTA0, GPIO_A, etc.
    ///   The app will alias those to friendlier names like swd_io, swdclk, which patterns and drivers will use.
    ///   So, I'd expect a hit in the alias HashMap more often than the in actual Pins', so check the alias first, then fall back to the pins.
    pub fn pin(&mut self, pin: &str) -> Option<&mut pin::Pin> {
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
        // for (n, a) in self.aliases.iter() {
        //     match a {
        //         AliasTypes::PinAlias(_a) => { retn.insert(n.clone(), _a.clone()); },
        //         AliasTypes::PinGroupAlias(_a) => {}
        //     }
        // }
        // for (n, a) in self.contents.iter() {
        //     match a {
        //         Types::Pin(_a) => { retn.insert(n.clone(), n.clone()); },
        //         Types::PinGroup(_a) => {}
        //     }
        // }
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

    pub fn group_pins(&mut self, name: &str, path: &str, pins: Vec<String>) -> Result<&mut pin::PinGroup, Error> {
        let n = name;
        if self.pin_group(name).is_some() {
            return Err(Error::new(&format!("Can not add pin group {} because it conflicts with a current pin group or alias name!", name)))
        }
        let p = pin::PinGroup::new(String::from(n), String::from(path), pins);
        self.pin_groups.insert(String::from(n), p);
        Ok(self.pin_groups.get_mut(n).unwrap())
    }

    pub fn pin_group(&mut self, name: &str) -> Option<&mut pin::PinGroup> {
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

    pub fn number_of_pin_groups(&mut self) -> usize {
        return self.pin_groups.len();
    }
}
