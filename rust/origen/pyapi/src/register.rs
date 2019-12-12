use origen::core::model::registers::Register;
use pyo3::prelude::*;

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
}

/// Rust-private methods, i.e. not accessible from Python
impl BitCollection {
    pub fn from_reg(
        path: &str,
        memory_map: Option<&str>,
        address_block: Option<&str>,
        reg: &Register,
    ) -> BitCollection {
        BitCollection {
            model_path: path.to_string(),
            memory_map: memory_map.unwrap_or("default").to_string(),
            address_block: address_block.unwrap_or("default").to_string(),
            reg_id: reg.id.clone(),
            whole: true,
            bit_numbers: Vec::new(),
        }
    }
}

/// Methods available from Rust and Python
#[pymethods]
impl BitCollection {}
