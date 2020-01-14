mod address_block;
mod bit_collection;
mod memory_map;
mod register;
use std::sync::RwLock;

//use crate::dut::PyDUT;
//use origen::core::model::registers::Register;
use bit_collection::BitCollection;
use origen::core::model::registers::{Bit, SummaryField};
use origen::DUT;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use register::{Field, FieldEnum};

#[pymodule]
/// Implements the module _origen.registers in Python
pub fn registers(_py: Python, m: &PyModule) -> PyResult<()> {
    // Used to pass register field info from Python to Rust when defining regs
    m.add_class::<Field>()?;
    m.add_class::<FieldEnum>()?;

    m.add_wrapped(wrap_pyfunction!(create))?;
    Ok(())
}

/// Create a new register
#[pyfunction]
fn create(
    address_block_id: usize,
    name: &str,
    offset: usize,
    size: Option<usize>,
    fields: Vec<&Field>,
) -> PyResult<usize> {
    let reg_id;
    let reg_fields;
    let base_bit_id;

    {
        base_bit_id = origen::dut().bits.len();
    }
    {
        let mut dut = origen::dut();
        reg_id = dut.create_reg(address_block_id, name, offset, size)?;
        let reg = dut.get_mut_register(reg_id)?;
        for f in &fields {
            let field = reg.add_field(
                &f.name,
                &f.description,
                f.offset,
                f.width,
                &f.access,
                &f.reset,
            )?;
            for e in &f.enums {
                field.add_enum(&e.name, &e.description, &e.value)?;
            }
        }
        for i in 0..reg.size as usize {
            reg.bit_ids.push((base_bit_id + i) as usize);
        }
        reg_fields = reg.named_bits(true).collect::<Vec<SummaryField>>();
    }

    // Create the bits now that we know which ones are implemented
    let mut dut = origen::dut();
    for field in reg_fields {
        for _i in 0..field.width {
            dut.bits.push(Bit {
                overlay: RwLock::new(None),
                register_id: reg_id,
                state: RwLock::new(0),
                access: field.access,
            });
        }
    }

    Ok(reg_id)
}

///// Returns an empty register collection
//fn empty_regs() -> PyResult<Registers> {
//    Ok(Registers {
//        address_block_id: None,
//        register_file_id: None,
//        ids: None,
//        i: 0,
//    })
//}

///// Implements the user APIs my_block.[.my_memory_map][.my_address_block].reg() and
///// my_block.[.my_memory_map][.my_address_block].regs
//#[pymethods]
//impl PyDUT {
//    fn regs(&self, address_block_id: Option<usize>) -> PyResult<Registers> {
//        Ok(Registers {
//            address_block_id: address_block_id,
//            i: 0,
//        })
//    }
//
//    fn reg(&self, address_block_id: usize, name: &str) -> PyResult<BitCollection> {
//        Ok(BitCollection {
//            reg_id: DUT
//                .lock()
//                .unwrap()
//                .get_address_block(address_block_id)?
//                .get_register_id(name)?,
//            whole: true,
//            bit_numbers: Vec::new(),
//            i: 0,
//        })
//    }
//}

/// Implements the user API to work with a collection of registers. The collection could be associated
/// with another container object (an address block or register file), or this could be its own collection
/// of otherwise un-related registers.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Registers {
    /// The ID of the address block which contains these registers. It is optional so that
    /// an empty Registers collection can be created, or a collection of
    pub address_block_id: Option<usize>,
    /// The ID of the register file which contains these registers. It is optional as registers
    /// can be instantiated in an address block directly and are not necessarily within an
    pub register_file_id: Option<usize>,
    /// The IDs of the contained registers. If not present then the IDs will be derived from either
    /// the associated register file or address block. If both are defined then the register file IDs
    /// will be used.
    pub ids: Option<Vec<usize>>,
    /// Iterator index
    pub i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl Registers {
    fn len(&self) -> PyResult<usize> {
        if self.address_block_id.is_some() {
            Ok(DUT
                .lock()
                .unwrap()
                .get_address_block(self.address_block_id.unwrap())?
                .registers
                .len())
        } else {
            Ok(0)
        }
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let keys: Vec<String> = ab.registers.keys().map(|x| x.clone()).collect();
            Ok(keys)
        } else {
            Ok(Vec::new())
        }
    }

    fn values(&self) -> PyResult<Vec<BitCollection>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let values: Vec<BitCollection> = ab
                .registers
                .values()
                .map(|x| BitCollection::from_reg_id(*x))
                .collect();
            Ok(values)
        } else {
            Ok(Vec::new())
        }
    }

    fn items(&self) -> PyResult<Vec<(String, BitCollection)>> {
        if self.address_block_id.is_some() {
            let dut = DUT.lock().unwrap();
            let ab = dut.get_address_block(self.address_block_id.unwrap())?;
            let items: Vec<(String, BitCollection)> = ab
                .registers
                .iter()
                .map(|(k, v)| (k.to_string(), BitCollection::from_reg_id(*v)))
                .collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }
}
