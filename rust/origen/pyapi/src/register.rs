use crate::dut::PyDUT;
//use origen::core::model::registers::Register;
use origen::DUT;
use pyo3::prelude::*;

/// Implements the user APIs my_block.[.my_memory_map][.my_address_block].reg() and
/// my_block.[.my_memory_map][.my_address_block].regs
#[pymethods]
impl PyDUT {
    fn regs(&self, address_block_id: usize) -> PyResult<Registers> {
        Ok(Registers {
            address_block_id: address_block_id,
            i: 0,
        })
    }

    fn reg(&self, address_block_id: usize, name: &str) -> PyResult<BitCollection> {
        Ok(BitCollection {
            reg_id: DUT
                .lock()
                .unwrap()
                .get_address_block(address_block_id)?
                .get_register_id(name)?,
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
    /// The ID of the address block which contains these registers
    pub address_block_id: usize,
    /// Iterator index
    pub i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl Registers {
    fn len(&self) -> PyResult<usize> {
        Ok(DUT
            .lock()
            .unwrap()
            .get_address_block(self.address_block_id)?
            .registers
            .len())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let ab = dut.get_address_block(self.address_block_id)?;
        let keys: Vec<String> = ab.registers.keys().map(|x| x.clone()).collect();
        Ok(keys)
    }

    fn values(&self) -> PyResult<Vec<BitCollection>> {
        let dut = DUT.lock().unwrap();
        let ab = dut.get_address_block(self.address_block_id)?;
        let values: Vec<BitCollection> = ab
            .registers
            .values()
            .map(|x| BitCollection::from_reg_id(*x))
            .collect();
        Ok(values)
    }

    fn items(&self) -> PyResult<Vec<(String, BitCollection)>> {
        let dut = DUT.lock().unwrap();
        let ab = dut.get_address_block(self.address_block_id)?;
        let items: Vec<(String, BitCollection)> = ab
            .registers
            .iter()
            .map(|(k, v)| (k.to_string(), BitCollection::from_reg_id(*v)))
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
    /// The ID of the parent register
    reg_id: usize,
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
    pub fn from_reg_id(id: usize) -> BitCollection {
        BitCollection {
            reg_id: id,
            whole: true,
            bit_numbers: Vec::new(),
            i: 0,
        }
    }
}

/// Methods available from Rust and Python
#[pymethods]
impl BitCollection {}
