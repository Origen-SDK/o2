use crate::dut::PyDUT;
use crate::register::Registers;
use origen::DUT;
use pyo3::class::basic::{CompareOp, PyObjectProtocol};
use pyo3::class::PyMappingProtocol;
use pyo3::exceptions::{AttributeError, KeyError, TypeError};
use pyo3::prelude::*;

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    fn create_memory_map(
        &self,
        model_id: usize,
        name: &str,
        address_unit_bits: Option<u32>,
    ) -> PyResult<usize> {
        Ok(DUT
            .lock()
            .unwrap()
            .create_memory_map(model_id, name, address_unit_bits)?)
    }

    fn get_or_create_address_block(&self, memory_map_id: usize, name: &str) -> PyResult<AddressBlock> {
        let mut dut = DUT.lock().unwrap();
        let mm = dut.get_memory_map(memory_map_id)?;
        let id = match model.get_memory_map_id(name) {
            Ok(v) => v,
            Err(_) => dut.create_memory_map(model_id, name, None)?,
        };
        Ok(MemoryMap {
            id: id,
            name: name.to_string(),
        })
    }
}
