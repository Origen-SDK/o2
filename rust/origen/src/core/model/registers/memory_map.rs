use crate::core::model::Model;
use crate::error::Error;
use crate::Dut;
use crate::Result as OrigenResult;
use indexmap::map::IndexMap;
use std::sync::MutexGuard;

#[derive(Debug)]
pub struct MemoryMap {
    pub name: String,
    pub id: usize,
    pub model_id: usize,
    /// Represents the number of bits of an address increment between two
    /// consecutive addressable units in the memory map.
    /// Its value defaults to 8 indicating a byte addressable memory map.
    pub address_unit_bits: u32,
    pub address_blocks: IndexMap<String, usize>,
}

impl Default for MemoryMap {
    fn default() -> MemoryMap {
        MemoryMap {
            id: 0,
            model_id: 0,
            name: "default".to_string(),
            address_unit_bits: 8,
            address_blocks: IndexMap::new(),
        }
    }
}

impl MemoryMap {
    /// Get the ID from the given address block name
    pub fn get_address_block_id(&self, name: &str) -> OrigenResult<usize> {
        match self.address_blocks.get(name) {
            Some(x) => Ok(*x),
            None => {
                return Err(Error::new(&format!(
                    "The memory map '{}' does not have an address block named '{}'",
                    self.name, name
                )))
            }
        }
    }

    /// Returns a path to this memory_map like "dut.my_block.my_map", but the map and address block portions
    /// will be inhibited when they are 'default'. This is to keep map and address block concerns out of the view of users who
    /// don't use them and simply define regs at the top-level of the block.
    pub fn friendly_path(&self, dut: &MutexGuard<Dut>) -> OrigenResult<String> {
        let path = self.model(dut)?.friendly_path(dut)?;
        if self.name == "default" {
            Ok(path)
        } else {
            Ok(format!("{}.{}", path, self.name))
        }
    }

    /// Returns an immutable reference to the parent model
    pub fn model<'a>(&self, dut: &'a MutexGuard<Dut>) -> OrigenResult<&'a Model> {
        dut.get_model(self.model_id)
    }

    pub fn console_display(&self, dut: &MutexGuard<Dut>) -> OrigenResult<String> {
        let (mut output, offset) = self.model(&dut)?.console_header(&dut);
        output += &(" ".repeat(offset));
        output += &format!("└── memory_maps['{}']\n", self.name);
        let mut leader = " ".repeat(offset + 5);
        output += &format!(
            "{}├── address_unit_bits: {}\n",
            leader, self.address_unit_bits
        );
        let num = self.address_blocks.keys().len();
        if num > 0 {
            output += &format!("{}└── address_blocks\n", leader);
            leader += "     ";
            for (i, key) in self.address_blocks.keys().enumerate() {
                if i != num - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}└── address_blocks []\n", leader);
        }
        Ok(output)
    }
}
