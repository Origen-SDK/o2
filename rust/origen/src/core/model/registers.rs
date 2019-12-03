//! See section 3.5.1 in this doc for a good description of the IP-XACT data
//! structures upon which this is based:
//! https://www.accellera.org/images/downloads/standards/ip-xact/IP-XACT_User_Guide_2018-02-16.pdf

use std::collections::HashMap;

#[derive(Debug)]
pub enum AccessType {
    ReadWrite,
    ReadOnly,
    WriteOnly,
    ReadWriteOnce,
    WriteOnce,
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
    /// Represents the number of bits of an address increment between two
    /// consecutive addressable units in the memory map.
    /// Its value defaults to 8 indicating a byte addressable memory map.
    pub address_unit_bits: u32,
    pub address_blocks: HashMap<String, AddressBlock>,
}

#[derive(Debug)]
/// Represents a single, contiguous block of memory in a memory map.
pub struct AddressBlock {
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
    pub registers: HashMap<String, Register>,
}

#[derive(Debug)]
pub struct Register {
    pub name: String,
    pub description: String,
    // TODO: What is this?!
    /// The dimension of the register, defaults to 1.
    pub dim: u16,
    /// Address offset from the start of the parent address block in address_unit_bits.
    pub offset: u16,
    /// The size of the register in bits.
    pub size: u16,
    pub access: AccessType,
    pub fields: HashMap<String, Field>,
    /// Contains all bits implemented by the register, bits[i] will return None if
    /// the bit is unimplemented/undefined
    pub bits: Vec<Bit>,
}

#[derive(Debug)]
/// Named collections of bits within a register
pub struct Field {
    pub name: String,
    pub description: String,
    /// Offset from the start of the register in bits.
    pub offset: u16,
    /// Width of the field in bits.
    pub width: u16,
    /// Contains any reset values defined for this field.
    pub resets: Vec<Reset>,
    pub enumerated_values: HashMap<String, EnumeratedValue>,
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
