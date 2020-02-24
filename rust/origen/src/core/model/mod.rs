pub mod pins;
pub mod registers;
pub mod timesets;
use crate::error::Error;
use crate::Dut;
use crate::Result;
use std::sync::MutexGuard;
use indexmap::map::IndexMap;

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
    pub pins: IndexMap<String, usize>,
    pub pin_groups: IndexMap<String, usize>,
    pub pin_headers: IndexMap<String, usize>,

    pub timesets: IndexMap<String, usize>,
    // TODO: Levels
    // TODO: Specs
    /// Represents the number of bits of an address increment between two
    /// consecutive addressable units within the block.
    /// Its value defaults to 8 indicating a byte addressable memory map.
    /// This attribute can be overridden by MemoryMaps defined within the block.
    /// Since this attribute is intrinsically linked to the definitions within block,
    /// it cannot be overridden by instantiation.
    pub address_unit_bits: u32,
    /// The starting address of the address block expressed in address_unit_bits
    /// from the parent block.
    /// This defaults to 0 but can be overridden by instantiation.
    pub base_address: u64,
}

impl Model {
    pub fn new(
        id: usize,
        name: String,
        parent_id: Option<usize>,
        base_address: Option<u64>,
    ) -> Model {
        Model {
            id: id,
            name: name,
            parent_id: parent_id,
            sub_blocks: IndexMap::new(),
            memory_maps: IndexMap::new(),
            pins: IndexMap::new(),
            pin_groups: IndexMap::new(),
            pin_headers: IndexMap::new(),
            timesets: IndexMap::new(),
            address_unit_bits: 8,
            base_address: match base_address {
                Some(x) => x,
                None => 0,
            },
        }
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
}
