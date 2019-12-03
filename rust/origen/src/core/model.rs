pub mod pins;
pub mod registers;

use registers::MemoryMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub memory_maps: HashMap<String, MemoryMap>,
}

impl Model {
    pub fn new(name: String) -> Model {
        Model {
            name: name,
            memory_maps: HashMap::new(),
        }
    }

    //pub fn add_reg(&mut self, name: &str, offset: u32) {
    //    let r = Reg {
    //        name: name.to_string(),
    //        offset: offset,
    //    };
    //    self.registers.insert(name.to_string(), r);
    //}

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
