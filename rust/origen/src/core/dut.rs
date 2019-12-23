use crate::core::model::registers::{AccessType, AddressBlock, Bit, MemoryMap, Register};
use crate::core::model::Model;
use crate::error::Error;
use crate::Result;

/// The DUT stores all objects associated with a particular device.
/// Each object type is organized into vectors, where a particular object's position within the
/// vector is considered its unique ID.
/// A register then (for example) does not embed its bit objects but rather contains a list of
/// bit IDs. This approach allows bits to be easily passed around by ID to enable the creation of
/// bit collections that are small (a subset of a register's bits) or very large (all bits in
/// a memory map).
#[derive(Debug)]
pub struct Dut {
    pub name: String,
    models: Vec<Model>,
    memory_maps: Vec<MemoryMap>,
    address_blocks: Vec<AddressBlock>,
    registers: Vec<Register>,
    bits: Vec<Bit>,
}

impl Dut {
    pub fn new(name: &str) -> Dut {
        // TODO: reserve some size for these?
        Dut {
            name: name.to_string(),
            models: Vec::<Model>::new(),
            memory_maps: Vec::<MemoryMap>::new(),
            address_blocks: Vec::<AddressBlock>::new(),
            registers: Vec::<Register>::new(),
            bits: Vec::<Bit>::new(),
        }
    }

    /// Change the DUT, this replaces the existing mode with a fresh one (i.e.
    /// deletes all current DUT metadata and state, and updates the name/ID field
    /// with the given value
    pub fn change(&mut self, name: &str) {
        self.name = name.to_string();
        self.models.clear();
        self.memory_maps.clear();
        self.address_blocks.clear();
        self.registers.clear();
        self.bits.clear();
        // Add the model for the DUT top-level (always ID 0)
        let _ = self.create_model(None, "dut");
    }

    /// Get a mutable reference to the model with the given ID
    pub fn get_mut_model(&mut self, id: usize) -> Result<&mut Model> {
        match self.models.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no model exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the model with the given ID, use get_mut_model if
    /// you need to modify it
    pub fn get_model(&self, id: usize) -> Result<&Model> {
        match self.models.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no model exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a mutable reference to the memory map with the given ID
    pub fn get_mut_memory_map(&mut self, id: usize) -> Result<&mut MemoryMap> {
        match self.memory_maps.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no memory_map exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the memory map with the given ID, use get_mut_memory_map if
    /// you need to modify it
    pub fn get_memory_map(&self, id: usize) -> Result<&MemoryMap> {
        match self.memory_maps.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no memory_map exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a mutable reference to the address block with the given ID
    pub fn get_mut_address_block(&mut self, id: usize) -> Result<&mut AddressBlock> {
        match self.address_blocks.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no address_block exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the address block with the given ID, use get_mut_address_block if
    /// you need to modify it
    pub fn get_address_block(&self, id: usize) -> Result<&AddressBlock> {
        match self.address_blocks.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no address_block exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a mutable reference to the register with the given ID
    pub fn get_mut_register(&mut self, id: usize) -> Result<&mut Register> {
        match self.registers.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no register exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the register with the given ID, use get_mut_register if
    /// you need to modify it
    pub fn get_register(&self, id: usize) -> Result<&Register> {
        match self.registers.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no register exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a mutable reference to the bit with the given ID
    pub fn get_mut_bit(&mut self, id: usize) -> Result<&mut Bit> {
        match self.bits.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no bit exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the bit with the given ID, use get_mut_bit if
    /// you need to modify it
    pub fn get_bit(&self, id: usize) -> Result<&Bit> {
        match self.bits.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no bit exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Create a new model adding it to the existing parent model with the given ID.
    /// The ID of the newly created model is returned to the caller who should save it
    /// if they want to access this model directly again (will also be accessible by name
    /// via the parent model).
    pub fn create_model(&mut self, parent_id: Option<usize>, name: &str) -> Result<usize> {
        let id;
        {
            id = self.models.len();
        }
        {
            if parent_id.is_some() {
                let m = self.get_mut_model(parent_id.unwrap())?;
                if m.sub_blocks.contains_key(name) {
                    return Err(Error::new(&format!(
                        "The block '{}' already contains a sub-block called '{}'",
                        m.display_path(), name
                    )));
                } else {
                    m.sub_blocks.insert(name.to_string(), id);
                }
            }
        }
        let new_model = Model::new(name.to_string(), parent_id);
        self.models.push(new_model);
        Ok(id)
    }

    pub fn create_memory_map(
        &mut self,
        model_id: usize,
        name: &str,
        address_unit_bits: Option<u32>,
    ) -> Result<usize> {
        let id;
        {
            id = self.memory_maps.len();
        }
        {
            let model = self.get_mut_model(model_id)?;

            if model.memory_maps.contains_key(name) {
                return Err(Error::new(&format!(
                    "The block '{}' already contains a memory map called '{}'",
                    model.name, name
                )));
            } else {
                model.memory_maps.insert(name.to_string(), id);
            }
        }

        let mut defaults = MemoryMap::default();
        match address_unit_bits {
            Some(v) => defaults.address_unit_bits = v,
            None => {}
        }
        self.memory_maps.push(MemoryMap {
            name: name.to_string(),
            ..defaults
        });
        Ok(id)
    }

    pub fn create_address_block(
        &mut self,
        memory_map_id: usize,
        name: &str,
        base_address: Option<u64>,
        range: Option<u64>,
        width: Option<u64>,
        access: Option<AccessType>,
    ) -> Result<usize> {
        let id;
        {
            id = self.address_blocks.len();
        }
        {
            let map = self.get_mut_memory_map(memory_map_id)?;

            if map.address_blocks.contains_key(name) {
                return Err(Error::new(&format!(
                    "The memory map '{}' already contains an address block called '{}'",
                    map.name, name
                )));
            } else {
                map.address_blocks.insert(name.to_string(), id);
            }
        }

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

        self.address_blocks.push(AddressBlock {
            name: name.to_string(),
            ..defaults
        });
        Ok(id)
    }

    pub fn create_reg(
        &mut self,
        address_block_id: usize,
        name: &str,
        offset: u32,
        size: Option<u32>,
    ) -> Result<usize> {
        let id;
        {
            id = self.registers.len();
        }
        {
            let a = self.get_mut_address_block(address_block_id)?;
            if a.registers.contains_key(name) {
                return Err(Error::new(&format!(
                    "The address block '{}' already contains a register called '{}'",
                    a.name, name
                )));
            } else {
                a.registers.insert(name.to_string(), id);
            }
        }
        let mut defaults = Register::default();
        match size {
            Some(v) => defaults.size = v,
            None => {}
        }
        let reg = Register {
            name: name.to_string(),
            offset: offset,
            ..defaults
        };

        //reg.create_bits();

        self.registers.push(reg);
        Ok(id)
    }
}
