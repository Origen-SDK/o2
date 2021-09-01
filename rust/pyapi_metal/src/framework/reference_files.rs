use crate::py_submodule;
use origen_metal::framework::reference_files as rf;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

#[pyfunction]
pub fn set_save_ref_dir(dir: &str) -> PyResult<()> {
    let dir = PathBuf::from(dir);
    rf::set_save_ref_dir(dir);
    Ok(())
}

#[pyfunction]
pub fn apply_ref(key: &str) -> PyResult<()> {
    Ok(rf::apply_ref(Path::new(key))?)
}

#[pyfunction]
pub fn apply_all_new_refs() -> PyResult<()> {
    Ok(rf::apply_all_new_refs()?)
}

#[pyfunction]
pub fn apply_all_changed_refs() -> PyResult<()> {
    Ok(rf::apply_all_changed_refs()?)
}

#[pyfunction]
pub fn create_changed_ref(key: &str, new_file: &str, ref_file: &str) -> PyResult<()> {
    Ok(rf::create_changed_ref(
        Path::new(key),
        Path::new(new_file),
        Path::new(ref_file),
    )?)
}

#[pyfunction]
pub fn create_new_ref(key: &str, new_file: &str, ref_file: &str) -> PyResult<()> {
    Ok(rf::create_new_ref(
        Path::new(key),
        Path::new(new_file),
        Path::new(ref_file),
    )?)
}

#[pyfunction]
pub fn clear_save_refs() -> PyResult<()> {
    Ok(rf::clear_save_refs()?)
}

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.framework.reference_files", |m| {
        m.add_function(wrap_pyfunction!(set_save_ref_dir, m)?)?;
        m.add_function(wrap_pyfunction!(apply_ref, m)?)?;
        m.add_function(wrap_pyfunction!(apply_all_new_refs, m)?)?;
        m.add_function(wrap_pyfunction!(apply_all_changed_refs, m)?)?;
        m.add_function(wrap_pyfunction!(create_changed_ref, m)?)?;
        m.add_function(wrap_pyfunction!(create_new_ref, m)?)?;
        m.add_function(wrap_pyfunction!(clear_save_refs, m)?)?;
        Ok(())
    })
}
