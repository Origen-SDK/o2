use crate::core::model::registers::{
    AccessType, AddressBlock, Bit, MemoryMap, Register, RegisterFile,
};
use crate::core::model::timesets::timeset::{Event, Timeset, Wave, WaveGroup, Wavetable};
use crate::core::model::Model;
use crate::error::Error;
use crate::meta::IdGetters;
use crate::Result;
use crate::DUT;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::RwLock;

/// The DUT stores all objects associated with a particular device.
/// Each object type is organized into vectors, where a particular object's position within the
/// vector is considered its unique ID.
/// A register then (for example) does not embed its bit objects but rather contains a list of
/// bit IDs. This approach allows bits to be easily passed around by ID to enable the creation of
/// bit collections that are small (a subset of a register's bits) or very large (all bits in
/// a memory map).
//#[include_id_getters]
#[derive(Debug, IdGetters)]
#[id_getters_by_mapping(
    field = "timeset",
    parent_field = "models",
    return_type = "Timeset",
    field_container_name = "timesets"
)]
#[id_getters_by_mapping(
    field = "wavetable",
    parent_field = "timesets",
    return_type = "Wavetable",
    field_container_name = "wavetables"
)]
#[id_getters_by_mapping(
    field = "wave_group",
    parent_field = "wavetables",
    return_type = "WaveGroup",
    field_container_name = "wave_groups"
)]
#[id_getters_by_mapping(
    field = "wave",
    parent_field = "wave_groups",
    return_type = "Wave",
    field_container_name = "waves"
)]
#[id_getters_by_index(
    field = "event",
    parent_field = "waves",
    return_type = "Event",
    field_container_name = "wave_events"
)]
pub struct Dut {
    pub name: String,
    models: Vec<Model>,
    memory_maps: Vec<MemoryMap>,
    address_blocks: Vec<AddressBlock>,
    register_files: Vec<RegisterFile>,
    registers: Vec<Register>,
    pub bits: Vec<Bit>,
    pub timesets: Vec<Timeset>,
    pub wavetables: Vec<Wavetable>,
    pub wave_groups: Vec<WaveGroup>,
    pub waves: Vec<Wave>,
    pub wave_events: Vec<Event>,
    pub id_mappings: Vec<IndexMap<String, usize>>,
}

impl Dut {
    // This is called only once at the start of an Origen thread to create the global database,
    // then the 'change' method is called every time a DUT is loaded
    pub fn new(name: &str) -> Dut {
        // TODO: reserve some size for these?
        Dut {
            name: name.to_string(),
            models: Vec::<Model>::new(),
            memory_maps: Vec::<MemoryMap>::new(),
            address_blocks: Vec::<AddressBlock>::new(),
            register_files: Vec::<RegisterFile>::new(),
            registers: Vec::<Register>::new(),
            bits: Vec::<Bit>::new(),
            timesets: Vec::<Timeset>::new(),
            wavetables: Vec::<Wavetable>::new(),
            wave_groups: Vec::<WaveGroup>::new(),
            waves: Vec::<Wave>::new(),
            wave_events: Vec::<Event>::new(),

            id_mappings: Vec::<IndexMap<String, usize>>::new(),
        }
    }

    /// Change the DUT, this replaces the existing mode with a fresh one (i.e.
    /// deletes all current DUT metadata and state, and updates the name/ID field
    /// with the given value
    // This is called once per DUT load
    pub fn change(&mut self, name: &str) {
        self.name = name.to_string();
        self.models.clear();
        self.memory_maps.clear();
        self.address_blocks.clear();
        self.register_files.clear();
        self.registers.clear();
        self.bits.clear();
        self.timesets.clear();
        self.wavetables.clear();
        self.wave_groups.clear();
        self.waves.clear();
        self.wave_events.clear();

        self.id_mappings.clear();
        // Add the model for the DUT top-level (always ID 0)
        let _ = self.create_model(None, "dut", None);
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

    /// Get a mutable reference to the register file with the given ID
    pub fn get_mut_register_file(&mut self, id: usize) -> Result<&mut RegisterFile> {
        match self.register_files.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no register_file exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the register file with the given ID, use get_mut_register_file if
    /// you need to modify it
    pub fn get_register_file(&self, id: usize) -> Result<&RegisterFile> {
        match self.register_files.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no register_file exists with ID '{}'",
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
    pub fn create_model(
        &mut self,
        parent_id: Option<usize>,
        name: &str,
        base_address: Option<u64>,
    ) -> Result<usize> {
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
                        m.display_path(&DUT.lock().unwrap()),
                        name
                    )));
                } else {
                    m.sub_blocks.insert(name.to_string(), id);
                }
            }
        }
        let new_model = Model::new(id, name.to_string(), parent_id, base_address);
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
            id: id,
            model_id: model_id,
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
            id: id,
            memory_map_id: memory_map_id,
            name: name.to_string(),
            ..defaults
        });
        Ok(id)
    }

    pub fn create_reg(
        &mut self,
        address_block_id: usize,
        name: &str,
        offset: usize,
        size: Option<usize>,
        bit_order: &str,
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
        match bit_order.parse() {
            Ok(x) => defaults.bit_order = x,
            Err(msg) => return Err(Error::new(&msg)),
        }
        let reg = Register {
            id: id,
            name: name.to_string(),
            offset: offset,
            ..defaults
        };

        self.registers.push(reg);
        Ok(id)
    }

    /// Creates a bit for testing bit collections and so on, does not add the new
    /// bit to a parent register
    pub fn create_test_bit(&mut self) -> usize {
        let id;
        {
            id = self.bits.len();
        }
        let bit = Bit {
            overlay: RwLock::new(None),
            overlay_snapshots: RwLock::new(HashMap::new()),
            register_id: 0,
            state: RwLock::new(0),
            reset_state: RwLock::new(0),
            device_state: RwLock::new(0),
            state_snapshots: RwLock::new(HashMap::new()),
            access: AccessType::ReadWrite,
        };

        self.bits.push(bit);
        id
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let dut = super::Dut::new("placeholder");
        //dut.get_event_test(0, 0);
        //dut.hello_macro();
        //assert_eq!(2 + 2, 4);
    }
}
