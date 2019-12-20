use crate::dut::PyDUT;
use origen::core::model::registers::Register;
use origen::DUT;
use pyo3::prelude::*;

/// Implements the user APIs my_block.[.my_memory_map][.my_address_block].reg() and
/// my_block.[.my_memory_map][.my_address_block].regs
#[pymethods]
impl PyDUT {
    fn regs(
        &self,
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
    ) -> PyResult<Registers> {
        // Verify the model exists, though we don't need it for now
        DUT.lock().unwrap().get_model(path)?;
        // TODO: Verify the memory_map and address_block
        Ok(Registers {
            model_path: path.to_string(),
            memory_map: memory_map.unwrap_or("default").to_string(),
            address_block: address_block.unwrap_or("default").to_string(),
            i: 0,
        })
    }

    fn reg(
        &self,
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        id: &str,
    ) -> PyResult<BitCollection> {
        // Verify the model exists, though we don't need it for now
        DUT.lock().unwrap().get_model(path)?;
        // TODO: Verify the memory_map and address_block
        Ok(BitCollection {
            model_path: path.to_string(),
            memory_map: memory_map.unwrap_or("default").to_string(),
            address_block: address_block.unwrap_or("default").to_string(),
            reg_id: id.to_string(),
            whole: true,
            bit_numbers: Vec::new(),
            i: 0,
        })
    }
}

/// Implements the user API to work with a model's collection of registers, an instance
/// of this is returned by my_block.[.my_memory_map][.my_address_block].regs
#[pyclass]
#[derive(Debug, Clone)]
pub struct Registers {
    /// The path to the model which owns the contained registers
    pub model_path: String,
    /// The name of the model's memory map which contains these registers
    pub memory_map: String,
    /// The name of the memory map's address block which contains these register
    pub address_block: String,
    /// Iterator index
    pub i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl Registers {
    fn len(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        let map = model.memory_maps.get(&self.memory_map).unwrap();
        let ab = map.address_blocks.get(&self.address_block).unwrap();
        Ok(ab.registers.len())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        let map = model.memory_maps.get(&self.memory_map).unwrap();
        let ab = map.address_blocks.get(&self.address_block).unwrap();
        let keys: Vec<String> = ab.registers.keys().map(|x| x.clone()).collect();
        Ok(keys)
    }

    fn values(&self) -> PyResult<Vec<BitCollection>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        let map = model.memory_maps.get(&self.memory_map).unwrap();
        let ab = map.address_blocks.get(&self.address_block).unwrap();
        let values: Vec<BitCollection> = ab
            .registers
            .keys()
            .map(|x| {
                BitCollection::from_reg(
                    &self.model_path,
                    &self.memory_map,
                    &self.address_block,
                    &ab.registers.get(x).unwrap(),
                )
            })
            .collect();
        Ok(values)
    }

    fn items(&self) -> PyResult<Vec<(String, BitCollection)>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        let map = model.memory_maps.get(&self.memory_map).unwrap();
        let ab = map.address_blocks.get(&self.address_block).unwrap();
        let items: Vec<(String, BitCollection)> = ab
            .registers
            .keys()
            .map(|x| {
                (
                    x.to_string(),
                    BitCollection::from_reg(
                        &self.model_path,
                        &self.memory_map,
                        &self.address_block,
                        &ab.registers.get(x).unwrap(),
                    ),
                )
            })
            .collect();
        Ok(items)
    }
}

/// A BitCollection represents either a whole register of a subset of a
/// registers bits (not necessarily contiguous bits) and provides the user
/// with the same API to set and consume register data in both cases.
#[pyclass]
#[derive(Debug)]
pub struct BitCollection {
    /// The path to the model which owns the parent register
    model_path: String,
    /// The name of the model's memory map which contains the register
    memory_map: String,
    /// The name of the memory map's address block which contains the register
    address_block: String,
    /// The ID of the parent register
    reg_id: String,
    /// When true the BitCollection contains an entire register's worth of bits
    whole: bool,
    /// The index numbers of the bits from the register that are included in this
    /// collection. Typically this will be mapped to the actual register bits by
    /// BitCollection's methods.
    bit_numbers: Vec<u16>,
    /// Iterator index
    i: usize,
}

/// Rust-private methods, i.e. not accessible from Python
impl BitCollection {
    pub fn from_reg(
        path: &str,
        memory_map: &str,
        address_block: &str,
        reg: &Register,
    ) -> BitCollection {
        BitCollection {
            model_path: path.to_string(),
            memory_map: memory_map.to_string(),
            address_block: address_block.to_string(),
            reg_id: reg.id.clone(),
            whole: true,
            bit_numbers: Vec::new(),
            i: 0,
        }
    }
}

/// Methods available from Rust and Python
#[pymethods]
impl BitCollection {}
