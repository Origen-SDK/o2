use super::{AccessType, MemoryMap};
use crate::core::model::Model;
use crate::Result as OrigenResult;
use crate::{Dut, Error};
use indexmap::map::IndexMap;
use std::sync::MutexGuard;

#[derive(Debug)]
/// Represents a single, contiguous block of memory in a memory map.
pub struct AddressBlock {
    pub id: usize,
    pub memory_map_id: usize,
    pub name: String,
    /// The starting address of the address block expressed in address_unit_bits
    /// from the parent memory map.
    pub offset: u128,
    /// The number of addressable units in the address block.
    pub range: u64,
    /// The maximum number of bits that can be accessed by a transaction into this
    /// address block.
    pub width: Option<u64>,
    pub access: AccessType,
    pub registers: IndexMap<String, usize>,
    pub register_files: IndexMap<String, usize>,
}

impl Default for AddressBlock {
    fn default() -> AddressBlock {
        AddressBlock {
            id: 0,
            memory_map_id: 0,
            name: "default".to_string(),
            offset: 0,
            range: 0,
            width: None,
            access: AccessType::RW,
            registers: IndexMap::new(),
            register_files: IndexMap::new(),
        }
    }
}

impl AddressBlock {
    /// Returns an immutable reference to the parent model
    pub fn model<'a>(&self, dut: &'a MutexGuard<Dut>) -> OrigenResult<&'a Model> {
        self.memory_map(dut)?.model(dut)
    }

    /// Returns an immutable reference to the parent memory map
    pub fn memory_map<'a>(&self, dut: &'a MutexGuard<Dut>) -> OrigenResult<&'a MemoryMap> {
        dut.get_memory_map(self.memory_map_id)
    }

    /// Returns the address_unit_bits size from the parent memory map
    pub fn address_unit_bits(&self, dut: &MutexGuard<Dut>) -> OrigenResult<u32> {
        Ok(self.memory_map(dut)?.address_unit_bits)
    }

    /// Returns the fully-resolved address taking into account all base addresses defined by the parent hierachy.
    /// The returned address is with an address_unit_bits size of 1.
    pub fn bit_address(&self, dut: &MutexGuard<Dut>) -> OrigenResult<u128> {
        let base = self.model(dut)?.bit_address(dut)?;
        Ok(base + (self.offset * self.address_unit_bits(dut)? as u128))
    }

    /// Returns a path to this address_block like "dut.my_block.my_map.my_address_block", but the map and address block portions
    /// will be inhibited when they are 'default'. This is to keep map and address block concerns out of the view of users who
    /// don't use them and simply define regs at the top-level of the block.
    pub fn friendly_path(&self, dut: &MutexGuard<Dut>) -> OrigenResult<String> {
        let path = self.memory_map(dut)?.friendly_path(dut)?;
        if self.name == "default" {
            Ok(path)
        } else {
            Ok(format!("{}.{}", path, self.name))
        }
    }

    /// Get the ID from the given register name
    pub fn get_register_id(&self, name: &str) -> OrigenResult<usize> {
        match self.registers.get(name) {
            Some(x) => Ok(*x),
            None => {
                return Err(Error::new(&format!(
                    "The address block '{}' does not have a register named '{}'",
                    self.name, name
                )))
            }
        }
    }

    pub fn console_display(&self, dut: &MutexGuard<Dut>) -> OrigenResult<String> {
        let (mut output, offset) = self.model(dut)?.console_header(dut);
        output += &(" ".repeat(offset));
        output += &format!("└── memory_maps['{}']\n", self.memory_map(dut)?.name);
        let mut leader = " ".repeat(offset + 5);
        output += &format!("{}└── address_blocks['{}']\n", leader, self.name);
        leader += "     ";
        let num = self.register_files.keys().len();
        if num > 0 {
            output += &format!("{}├── register_files\n", leader);
            let leader = format!("{}|    ", leader);
            for (i, key) in self.register_files.keys().enumerate() {
                if i != num - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}├── register_files []\n", leader);
        }
        let num = self.registers.keys().len();
        if num > 0 {
            output += &format!("{}└── registers\n", leader);
            let leader = format!("{}     ", leader);
            for (i, key) in self.registers.keys().enumerate() {
                if i != num - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}├── registers []\n", leader);
        }
        Ok(output)
    }
}
