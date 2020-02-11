use super::AddressBlock;
use crate::Dut;
use crate::Result;
use indexmap::map::IndexMap;
use std::sync::MutexGuard;

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
    pub offset: u128,
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
            offset: 0,
            range: 0,
            registers: IndexMap::new(),
            register_files: IndexMap::new(),
        }
    }
}

impl RegisterFile {
    /// Returns an immutable reference to the address block object that owns the register file.
    /// Note that this may or may not be the immediate parent of the register file depending on
    /// whether it is instantiated within another register file or not.
    pub fn address_block<'a>(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a AddressBlock> {
        dut.get_address_block(self.address_block_id)
    }

    /// Returns an immutable reference to the register file object that owns the register file.
    /// If it returns None it means that the register file is instantiated directly within an
    /// address block.
    pub fn register_file<'a>(&self, dut: &'a MutexGuard<Dut>) -> Option<Result<&'a RegisterFile>> {
        match self.register_file_id {
            Some(x) => Some(dut.get_register_file(x)),
            None => None,
        }
    }

    /// Returns the address_unit_bits size that the register file's offset is defined in.
    pub fn address_unit_bits(&self, dut: &MutexGuard<Dut>) -> Result<u32> {
        match self.register_file(dut) {
            Some(x) => Ok(x?.address_unit_bits(dut)?),
            None => Ok(self.address_block(dut)?.address_unit_bits(dut)?),
        }
    }

    /// Returns the fully-resolved address taking into account all base addresses defined by the parent hierachy.
    /// The returned address is with an address_unit_bits size of 1.
    pub fn bit_address(&self, dut: &MutexGuard<Dut>) -> Result<u128> {
        let base = match self.register_file_id {
            Some(_x) => self.register_file(dut).unwrap()?.bit_address(dut)?,
            None => self.address_block(dut)?.bit_address(dut)?,
        };
        Ok(base + (self.offset * self.address_unit_bits(dut)? as u128))
    }
}
