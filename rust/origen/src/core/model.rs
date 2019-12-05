pub mod pins;
pub mod registers;

use registers::{AccessType, AddressBlock, MemoryMap, Register};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub id: String,
    pub memory_maps: HashMap<String, MemoryMap>,
}

impl Model {
    pub fn new(id: String) -> Model {
        Model {
            id: id,
            memory_maps: HashMap::new(),
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

    pub fn add_reg(
        &mut self,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
        offset: u32,
        size: Option<u32>,
    ) {
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
        ab.registers.insert(
            id.to_string(),
            Register {
                id: id.to_string(),
                offset: offset,
                ..defaults
            },
        );
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
