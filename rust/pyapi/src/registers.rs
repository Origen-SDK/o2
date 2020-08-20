mod address_block;
pub mod bit_collection;
mod memory_map;
mod register;
mod register_collection;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use register::{Field, FieldEnum, ResetVal};

pub use register_collection::RegisterCollection;
use origen::core::model::registers::register::Register;

#[pymodule]
/// Implements the module _origen.registers in Python
pub fn registers(_py: Python, m: &PyModule) -> PyResult<()> {
    // Used to pass register field info from Python to Rust when defining regs
    m.add_class::<Field>()?;
    m.add_class::<FieldEnum>()?;
    m.add_class::<ResetVal>()?;

    m.add_wrapped(wrap_pyfunction!(create))?;
    Ok(())
}

/// Create a new register, returning its ID
#[pyfunction]
fn create(
    address_block_id: usize,
    register_file_id: Option<usize>,
    name: &str,
    offset: usize,
    size: Option<usize>,
    bit_order: String,
    fields: Vec<PyRef<Field>>,
    filename: Option<String>,
    lineno: Option<usize>,
    description: Option<String>,
    resets: Option<Vec<PyRef<ResetVal>>>,
    access: Option<&str>,
) -> PyResult<usize> {
    let r;
    if let Some(_resets) = resets {
        r = Some(_resets.iter().map( |res| res.to_origen_reset_val()).collect());
    } else {
        r = None
    }
    let mut dut = origen::dut();
    Ok(Register::add_reg(
        &mut dut,
        address_block_id,
        register_file_id,
        name,
        offset,
        size,
        &bit_order,
        filename,
        lineno,
        description,
        access,
        r,
        fields.iter().map( |f| f.to_origen_field()).collect()
    )?)
}
