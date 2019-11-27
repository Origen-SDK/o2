pub mod registers;
pub mod pins;

use registers::Reg;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    name: String,
    // TODO: This should be richer, more like an IP-XACT memory map block
    registers: HashMap<String, Reg>,
}

impl Model {
    pub fn new(name: String) -> Model {
        Model {
            name: name,
            registers: HashMap::new(),
        }
    }

    pub fn add_reg(&mut self, name: &str, offset: u32) {
        let r = Reg{ name: name.to_string(), offset: offset };
        self.registers.insert(name.to_string(), r);
    }

    pub fn number_of_regs(&self) -> u32 {
        self.registers.len() as u32
    }
}
