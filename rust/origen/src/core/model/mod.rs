pub mod pins;
pub mod registers;
pub mod timesets;
use crate::error::Error;
use crate::Dut;
use crate::Result;
use std::sync::MutexGuard;

use indexmap::map::IndexMap;
use pins::pin::Pin;
use pins::pin_group::PinGroup;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub id: usize,
    pub name: String,
    /// The only one without a parent is the top-level DUT model
    pub parent_id: Option<usize>,
    /// All children of this block/model, which are themselves models
    pub sub_blocks: IndexMap<String, usize>,
    /// All registers owned by this model are arranged within memory maps
    pub memory_maps: IndexMap<String, usize>,
    // Pins
    pub physical_pins: HashMap<String, Pin>,
    pub pins: HashMap<String, PinGroup>,
    pub timesets: IndexMap<String, usize>,
    // TODO: Levels
    // TODO: Specs
    /// Represents the number of bits in an address increment between two
    /// consecutive addressable units within the block.
    /// Its value defaults to 8 indicating that the address offsets of its sub-blocks
    /// will be expressed as byte addresses.
    /// This attribute can be overridden by individual MemoryMaps defined within the block.
    /// Since this attribute is intrinsically linked to the address definitions made
    /// within the block, it cannot be overridden by instantiation.
    pub address_unit_bits: u32,
    /// The starting address of the block expressed in address_unit_bits of the parent block.
    /// This defaults to 0 but can be overridden by instantiation.
    pub offset: u128,
    pub services: IndexMap<String, usize>,
}

impl Model {
    pub fn new(id: usize, name: String, parent_id: Option<usize>, offset: Option<u128>) -> Model {
        Model {
            id: id,
            name: name,
            parent_id: parent_id,
            sub_blocks: IndexMap::new(),
            memory_maps: IndexMap::new(),
            physical_pins: HashMap::new(),
            pins: HashMap::new(),
            timesets: IndexMap::new(),
            address_unit_bits: 8,
            offset: match offset {
                Some(x) => x,
                None => 0,
            },
            services: IndexMap::new(),
        }
    }

    pub fn add_service(&mut self, name: &str, id: usize) -> Result<()> {
        if self.services.contains_key(name) {
            return Err(Error::new(&format!(
                "The model '{}' already has a service called '{}'",
                self.name, name
            )));
        } else {
            self.services.insert(name.to_string(), id);
        }
        Ok(())
    }

    pub fn lookup(&self, key: &str) -> Result<&IndexMap<String, usize>> {
        match key {
            "timesets" => Ok(&self.timesets),
            _ => Err(Error::new(&format!(
                "No ID lookup table available for {}",
                key
            ))),
        }
    }

    /// Returns the hierarchical name of the model and the offset for console displays
    pub fn console_header(&self, dut: &MutexGuard<Dut>) -> (String, usize) {
        let l = format!("{}", self.display_path(dut));
        let mut names: Vec<&str> = l.split(".").collect();
        names.pop();
        if names.is_empty() {
            (l + "\n", 1)
        } else {
            let s = names.join(".").chars().count() + 2;
            (l + "\n", s)
        }
    }

    /// Get the ID for the given memory map name, throw an error if it doesn't exist
    pub fn get_memory_map_id(&self, name: &str) -> Result<usize> {
        match self.memory_maps.get(name) {
            Some(x) => Ok(*x),
            None => {
                return Err(Error::new(&format!(
                    "The block '{}' does not have a memory map named '{}'",
                    self.name, name
                )))
            }
        }
    }

    /// Returns the path to this model for displaying to a user, e.g. in error messages.
    pub fn display_path(&self, dut: &MutexGuard<Dut>) -> String {
        match self.parent_id {
            Some(p) => {
                let parent = dut.get_model(p).unwrap();
                return format!("{}.{}", parent.display_path(dut), self.name);
            }
            None => return format!("{}", self.name),
        }
    }

    pub fn console_display(&self, dut: &MutexGuard<Dut>) -> Result<String> {
        let (mut output, offset) = self.console_header(&dut);
        let offset = " ".repeat(offset);
        let num = self.memory_maps.keys().len();
        if num > 0 {
            output += &format!("{}├── memory_maps\n", offset);
            let leader = format!("{}|    ", offset);
            for (i, key) in self.memory_maps.keys().enumerate() {
                if i != num - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}├── memory_maps []\n", offset);
        }
        let num = self.sub_blocks.keys().len();
        if num > 0 {
            output += &format!("{}└── sub_blocks\n", offset);
            let leader = format!("{}     ", offset);
            for (i, key) in self.sub_blocks.keys().enumerate() {
                if i != num - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}└── sub_blocks []\n", offset);
        }
        Ok(output)
    }

    /// Returns the parent of this model or None, normally meaning that this is the top-level model
    pub fn parent<'a>(&self, dut: &'a MutexGuard<Dut>) -> Result<Option<&'a Model>> {
        match self.parent_id {
            Some(p) => return Ok(Some(dut.get_model(p)?)),
            None => return Ok(None),
        }
    }

    /// Returns the fully resolved address of the block which is comprised of the sum of
    /// it's own offset and that of it's parent(s).
    pub fn address(&self, dut: &MutexGuard<Dut>) -> Result<u128> {
        match self.parent(dut)? {
            Some(p) => return Ok(p.address(dut)? + self.offset),
            None => return Ok(self.offset),
        }
    }

    /// Returns the fully-resolved address taking into account all base addresses defined by the parent hierachy.
    /// The returned address is with an address_unit_bits size of 1.
    pub fn bit_address(&self, dut: &MutexGuard<Dut>) -> Result<u128> {
        match self.parent(dut)? {
            None => Ok(0),
            Some(x) => {
                let base = x.bit_address(dut)?;
                Ok(base + (self.offset * x.address_unit_bits as u128))
            }
        }
    }

    /// Returns a path to this block like "dut.my_block"
    pub fn friendly_path(&self, dut: &MutexGuard<Dut>) -> Result<String> {
        match self.parent(dut)? {
            None => Ok("dut".to_string()),
            Some(x) => Ok(format!("{}.{}", x.friendly_path(dut)?, self.name)),
        }
    }
}
