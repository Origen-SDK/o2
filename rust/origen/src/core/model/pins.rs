pub mod pin;
pub mod pin_collection;
use crate::error::Error;

use std::collections::HashMap;
use pin_collection::{PinCollection, Endianness};

/*
The most efficient way to store pins, pin groups, and their aliases, would be a single HashMap that holds Pins, PinGroups,
and aliases to each. When an alias is given, it will lookup the actual value and return that. The first part could be done using an enum, 
but the second angers the borrower checker and requires unsafe code to work around.

The above would work because we know the portion that contains the aliases are separate from the portion that contains the actual pins/groups.
Unfortanutely, Rust's HashMap doesn't have a way to tell it that, like it does for vector slicing, so we'll need to maintain two HashMaps:
one for the actual values, and one for aliases. 
*/

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
    pub groups: HashMap<String, PinCollection>,
    pub group_aliases: HashMap<String, String>,

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
            groups: HashMap::new(),
            group_aliases: HashMap::new(),
        }
    }

    pub fn add_pin(&mut self, name: &str) -> Result<&mut pin::Pin, Error> {
        let n = name;
        let p = pin::Pin::new(String::from(n));
        self.pins.insert(String::from(n), p);
        Ok(self.pins.get_mut(n).unwrap())
    }

    /// Returns the phyiscal pin, or None, if the pin doesn't exist.
    /// Implementation note: based on anecdotal evidence, physical pins generally have hardware-oriented pins, e.g.: PTA0, GPIO_A, etc.
    ///   The app will alias those to friendlier names like swd_io, swdclk, which patterns and drivers will use.
    ///   So, I'd expect a hit in the alias HashMap more often than the in actual Pins', so check the alias first, then fall back to the pins.
    pub fn get_pin(&mut self, pin: &str) -> Option<&mut pin::Pin> {
        let _p = pin;
        if let Some(p) = self.pin_aliases.get(pin) {
            let _p = p;
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

/*
    // / Resolve the pin or pin group given either a pin/group name, or pin/group alias.
    // / Note the enumerable here allowing us to return either Pin or a PinCollection.
    // / Also note, this function assumes proper PinContainer usage, that is, it's not going to check that
    // /     one and only return value is possible.
    // pub fn resolve(&mut self, query: &str) -> Result<&mut PinOrGroup, Error> {
    //     // Lookup order here is based on anecdotal evidence -> pin names are usually hardware oriented (e.g, PTA0, PTA1) but referenced to friendly names,
    //     //   such as TCLK, or TDO. Start with lookups in the alias
    //     // The alternative here is to have a single structure of all names which carry an enumerabe of what its name is and what it points to, which
    //     //   resolves from a if-else block and results in.. another lookup! Easiest to just brute force it and check each Hash table for the key in turn.

    // }

    // /// Aliases one pin to another pin, returning Ok if the alias occurred, or an error, if not.
    // /// Error conditions could be:
    // ///     * Pin doesn't exists.
    // ///     * Alias already exists as either a pin, alias to pin, group, or alias to a group.
    // pub fn add_pin_alias(&mut self, pin: &str, alias: &str) -> Result<())> {
    //     if let Some(p) = self.pins.get_mut(pin) {
    //         // Pin exists. Add the alias.
    //         ...
    //     } else {
    //         // Pin doesn't exists. Complain about this.
    //         Err(Error::new(&format("Could not alias pin {} to {}, as pin {} doesn't exists!", alias, pin)))
    //     }
    // }

    pub fn group_pins(&mut self, name: &str, pins: Vec<&str>, endianness: Endianness) -> Result<(), Error> {
        let p = PinCollection::new(self, &pins, Option::Some(endianness));
        self.contents.insert(String::from(name), Types::PinGroup(p));
        Ok(())
    }

    pub fn pin_group(&mut self, name: &str) -> Option<&mut PinCollection> {
        if let Some(a) = self.aliases.get(name) {
            match (a) {
                AliasTypes::PinAlias(_a) => Option::None, //Err(Error::new(&format!("No pin {} available!", name))),
                AliasTypes::PinGroupAlias(_a) => {
                    if let Some(_n) = self.contents.get_mut(name) {
                        match(_n) {
                            Types::Pin(_p) => None, // Err(Error::new(&format!("No pin {} available!", name))),
                            Types::PinGroup(_p) => Option::Some(_p)
                        }
                    } else {
                        Option::None // Err(Error::new(&format!("No pin {} available!", name)))
                    }
                },
                _ => Option::None // Err(Error::new(&format!("No pin {} available!", name))),
            }
        } else {
            // Not an alias. Try the contents directly.
            if let Some(_n) = self.contents.get_mut(name) {
                match(_n) {
                    Types::Pin(_p) => Option::None, // Err(Error::new(&format!("No pin {} available!", name))),
                    Types::PinGroup(_p) => Option::Some(_p)
                }
            } else {
                Option::None // Err(Error::new(&format!("No pin {} available!", name)))
            }
        }
    }
*/
}
