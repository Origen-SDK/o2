use indexmap::map::IndexMap;

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
