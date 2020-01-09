use crate::dut::PyDUT;
//use crate::register::Registers;
use origen::DUT;
//use pyo3::class::basic::{CompareOp, PyObjectProtocol};
//use pyo3::class::PyMappingProtocol;
//use pyo3::exceptions::{AttributeError, KeyError, TypeError};
use pyo3::prelude::*;

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    //fn create_address_block(
    //    &self,
    //    memory_map_id: usize,
    //    name: &str,
    //    base_address: Option<u64>,
    //    range: Option<u64>,
    //    width: Option<u64>,
    //    access: Option<&str>,
    //) -> PyResult<usize> {
    //    let acc: AccessType = match access {
    //        Some(x) => match x.parse() {
    //            Ok(y) => y,
    //            Err(msg) => return Err(exceptions::OSError::py_err(msg)),
    //        },
    //        None => AccessType::ReadWrite,
    //    };

    //    Ok(DUT.lock().unwrap().create_address_block(
    //        memory_map_id,
    //        name,
    //        base_address,
    //        range,
    //        width,
    //        Some(acc),
    //    )?)
    //}

    fn get_or_create_address_block(
        &self,
        memory_map_id: usize,
        name: &str,
    ) -> PyResult<AddressBlock> {
        let mut dut = DUT.lock().unwrap();
        let mm = dut.get_memory_map(memory_map_id)?;
        let id = match mm.get_address_block_id(name) {
            Ok(v) => v,
            Err(_) => dut.create_address_block(memory_map_id, name, None, None, None, None)?,
        };
        Ok(AddressBlock {
            id: id,
            name: name.to_string(),
        })
    }
}

/// Implements the user API to work with a single address block
#[pyclass]
#[derive(Debug)]
pub struct AddressBlock {
    #[pyo3(get)]
    id: usize,
    #[pyo3(get)]
    name: String,
}
