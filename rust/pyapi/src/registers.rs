mod address_block;
pub mod bit_collection;
mod memory_map;
mod register;
mod register_collection;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use register::{Field, FieldEnum, ResetVal};

use origen::core::model::registers::register::Register;
pub use register_collection::RegisterCollection;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "registers")?;
    subm.add_class::<Field>()?;
    subm.add_class::<FieldEnum>()?;
    subm.add_class::<ResetVal>()?;
    subm.add_wrapped(wrap_pyfunction!(create))?;
    m.add_submodule(subm)?;
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
        r = Some(
            _resets
                .iter()
                .map(|res| res.to_origen_reset_val())
                .collect(),
        );
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
        fields.iter().map(|f| f.to_origen_field()).collect(),
    )?)
}
