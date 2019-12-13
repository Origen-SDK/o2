pub mod pins;
pub mod registers;
use crate::error::Error;
use crate::Result;

use registers::{AccessType, AddressBlock, MemoryMap, Register};
use pins::{PinContainer};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub id: String,
    /// Store a hierarchical reference to the parent model minus the leading 'dut',
    /// e.g. if the given sub-block associated with this model was instantiated as
    /// "dut.core0.ana.adc0" then the id would be "adc0" and the parent would be
    /// "core0.ana".
    /// This means that the model can be identified as the top-level if parent == "" and
    /// also a model's parent can be found by fetching if from the DUT.
    /// Other approaches by trying to store a direct reference to the parent object just
    /// seem too scary in Rust, albeit a bit more efficient.
    pub parent_path: String,
    /// Returns the path to this model for displaying to a user, e.g. in error messages.
    pub display_path: String,
    /// All children of this block/model, which are themselves models
    pub sub_blocks: HashMap<String, Model>,
    /// All registers owned by this model are arranged within memory maps
    pub memory_maps: HashMap<String, MemoryMap>,
    /// Custom PinContainer block 
    pub pin_container: PinContainer,
    // Levels
    // Timing
    // Specs
}

impl Model {
    pub fn new(id: String, parent_path: String) -> Model {
        let mut p = "dut".to_string();
        if parent_path != "" {
            p = format!("{}.{}", p, parent_path);
        }
        if id != "" {
            p = format!("{}.{}", p, id);
        }
        Model {
            id: id,
            parent_path: parent_path,
            display_path: p,
            sub_blocks: HashMap::new(),
            memory_maps: HashMap::new(),
            pin_container: PinContainer::new(),
        }
    }

    pub fn add_memory_map(&mut self, id: &str, address_unit_bits: Option<u32>) {
        let mut defaults = MemoryMap::default();
        match address_unit_bits {
            Some(v) => defaults.address_unit_bits = v,
            None => {}
        }
        self.memory_maps.insert(
            id.to_string(),
            MemoryMap {
                id: id.to_string(),
                ..defaults
            },
        );
    }

    pub fn add_address_block(
        &mut self,
        memory_map_id: &str,
        id: &str,
        base_address: Option<u64>,
        range: Option<u64>,
        width: Option<u64>,
        access: Option<AccessType>,
    ) {
        let mut defaults = AddressBlock::default();
        match base_address {
            Some(v) => defaults.base_address = v,
            None => {}
        }
        match range {
            Some(v) => defaults.range = v,
            None => {}
        }
        match width {
            Some(v) => defaults.width = v,
            None => {}
        }
        match access {
            Some(v) => defaults.access = v,
            None => {}
        }
        if let Some(map) = self.memory_maps.get_mut(memory_map_id) {
            map.address_blocks.insert(
                id.to_string(),
                AddressBlock {
                    id: id.to_string(),
                    ..defaults
                },
            );
        } else {
            panic!("Tried to add address block '{}' to memory map '{}' but the memory map does not exist!", id, memory_map_id);
        }
    }

    pub fn create_reg(
        &mut self,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
        offset: u32,
        size: Option<u32>,
    ) -> Result<()> {
        let map_id = memory_map.unwrap_or("Default");
        let ab_id = address_block.unwrap_or("Default");

        // Create the memory map if it doesn't exist
        if !self.memory_maps.contains_key(map_id) {
            self.add_memory_map(map_id, None);
        }
        // Create the address block if it doesn't exist
        let exists;
        {
            exists = self
                .memory_maps
                .get(map_id)
                .unwrap()
                .address_blocks
                .contains_key(ab_id);
        }
        if !exists {
            self.add_address_block(map_id, ab_id, None, None, None, None);
        }

        // Now build the register
        let mut defaults = Register::default();
        match size {
            Some(v) => defaults.size = v,
            None => {}
        }

        let map = self.memory_maps.get_mut(map_id).unwrap();
        let ab = map.address_blocks.get_mut(ab_id).unwrap();

        if ab.registers.contains_key(id) {
            return Err(Error::new(&format!(
                "The block '{}' already contains a register called '{}' in address block {}.{}",
                self.display_path, id, map_id, ab_id
            )));
        } else {
            ab.registers.insert(
                id.to_string(),
                Register {
                    id: id.to_string(),
                    offset: offset,
                    ..defaults
                },
            );
        }
        Ok(())
    }

    pub fn number_of_regs(&self) -> usize {
        let mut count = 0;
        for (_k, v) in self.memory_maps.iter() {
            for (_k, v) in v.address_blocks.iter() {
                count += v.registers.len();
            }
        }
        count
    }
}
