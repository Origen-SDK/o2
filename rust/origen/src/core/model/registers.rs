//! See section 3.5.1 in this doc for a good description of the IP-XACT data
//! structures upon which this is based:
//! https://www.accellera.org/images/downloads/standards/ip-xact/IP-XACT_User_Guide_2018-02-16.pdf

use crate::core::model::Model;
use crate::error::Error;
use crate::Dut;
use crate::Result as OrigenResult;
use indexmap::map::IndexMap;
use std::sync::MutexGuard;

#[derive(Debug)]
pub enum AccessType {
    ReadWrite,
    ReadOnly,
    WriteOnly,
    ReadWriteOnce,
    WriteOnce,
}

impl std::str::FromStr for AccessType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ReadWrite" => Ok(AccessType::ReadWrite),
            "ReadOnly" => Ok(AccessType::ReadOnly),
            "WriteOnly" => Ok(AccessType::WriteOnly),
            "ReadWriteOnce" => Ok(AccessType::ReadWriteOnce),
            "WriteOnce" => Ok(AccessType::WriteOnce),
            _ => Err(format!("'{}' is not a valid value for AccessType", s)),
        }
    }
}

#[derive(Debug)]
pub enum Usage {
    Read,
    Write,
    ReadWrite,
}

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
            name: "Default".to_string(),
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

#[derive(Debug)]
/// Represents a single, contiguous block of memory in a memory map.
pub struct AddressBlock {
    pub id: usize,
    pub memory_map_id: usize,
    pub name: String,
    /// The starting address of the address block expressed in address_unit_bits
    /// from the parent memory map.
    pub base_address: u64,
    /// The number of addressable units in the address block.
    pub range: u64,
    /// The maximum number of bits that can be accessed by a transaction into this
    /// address block.
    pub width: u64,
    pub access: AccessType,
    pub registers: IndexMap<String, usize>,
    pub register_files: IndexMap<String, usize>,
}

impl Default for AddressBlock {
    fn default() -> AddressBlock {
        AddressBlock {
            id: 0,
            memory_map_id: 0,
            name: "Default".to_string(),
            base_address: 0,
            range: 0,
            width: 0,
            access: AccessType::ReadWrite,
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

#[derive(Debug)]
/// Represents a groups of registers within an address block. RegisterFiles can also contain
/// other RegisterFiles.
pub struct RegisterFile {
    pub id: usize,
    pub address_block_id: usize,
    /// Optional, if this register file is a child of another register file then its parent ID
    /// will be recorded here
    pub register_file_id: Option<usize>,
    pub name: String,
    pub description: String,
    // TODO: What is this?!
    /// The dimension of the register, defaults to 1.
    pub dim: u32,
    /// The address offset from the containing address block or register file,
    /// expressed in address_unit_bits from the parent memory map.
    pub address_offset: u64,
    /// The number of addressable units in the register file.
    pub range: u64,
    pub registers: IndexMap<String, usize>,
    pub register_files: IndexMap<String, usize>,
}

impl Default for RegisterFile {
    fn default() -> RegisterFile {
        RegisterFile {
            id: 0,
            address_block_id: 0,
            register_file_id: None,
            name: "Default".to_string(),
            description: "".to_string(),
            dim: 1,
            address_offset: 0,
            range: 0,
            registers: IndexMap::new(),
            register_files: IndexMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Register {
    pub name: String,
    pub description: String,
    // TODO: What is this?!
    /// The dimension of the register, defaults to 1.
    pub dim: u32,
    /// Address offset from the start of the parent address block in address_unit_bits.
    pub offset: u32,
    /// The size of the register in bits.
    pub size: u32,
    pub access: AccessType,
    pub fields: IndexMap<String, Field>,
    /// Contains all bits implemented by the register, bits[i] will return None if
    /// the bit is unimplemented/undefined
    pub bits: Vec<Bit>,
}

impl Default for Register {
    fn default() -> Register {
        Register {
            name: "Default".to_string(),
            description: "".to_string(),
            dim: 1,
            offset: 0,
            size: 32,
            access: AccessType::ReadWrite,
            fields: IndexMap::new(),
            bits: Vec::new(),
        }
    }
}

impl Register {
    pub fn create_bits(&mut self) {
        for _i in 0..self.size {
            self.bits.push(Bit::default());
        }
    }
}

#[derive(Debug)]
/// Named collections of bits within a register
pub struct Field {
    pub name: String,
    pub description: String,
    /// Offset from the start of the register in bits.
    pub offset: u32,
    /// Width of the field in bits.
    pub width: u32,
    /// Contains any reset values defined for this field.
    pub resets: Vec<Reset>,
    pub enumerated_values: IndexMap<String, EnumeratedValue>,
}

#[derive(Debug)]
pub struct Reset {
    pub reset_type: String,
    // TODO: Should this be vector of tuples instead?
    /// The size of this vector corresponds to the size of the parent field.
    /// A set bit indicates a reset values of 1.
    pub value: Vec<bool>,
    /// The size of this vector corresponds to the size of the parent field.
    /// A set bit indicates that the bit has a reset value defined by the
    /// corresponding value.
    pub mask: Vec<bool>,
}

#[derive(Debug)]
pub struct EnumeratedValue {
    pub name: String,
    pub description: String,
    pub usage: Usage,
    /// The size of this vector corresponds to the size of the parent field.
    /// A set bit indicates a value of 1.
    pub value: Vec<bool>,
}

#[derive(Debug)]
pub struct Bit {
    /// When true the bit stores a 1, else 0 (unless the Z or X bit is set)
    pub set: bool,
    /// When set the bit value is X
    pub x: bool,
    /// When set the bit value is Z
    pub z: bool,
    /// When set the overlay string should be applied to pattern vectors for this bit
    pub overlay: bool,
    pub overlay_str: String,
    /// When set the bit should be compared during a read transaction
    pub compare: bool,
    /// When set the bit should be captured during a read transaction
    pub capture: bool,
}

impl Default for Bit {
    fn default() -> Bit {
        Bit {
            set: false,
            x: false,
            z: false,
            overlay: false,
            overlay_str: "".to_string(),
            compare: false,
            capture: false,
        }
    }
}
