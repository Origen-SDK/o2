use crate::py_submodule;
use origen_metal::utils::differ::{ASCIIDiffer, Differ};
use pyo3::prelude::*;
use std::path::Path;

/// This function compares the given two files a
#[pyfunction(ignore_blank_lines = "true")]
//pub fn has_diffs(file_a: &str, file_b: &str, comment_chars: Vec<&str>, suspend_on: Option<&str>, resume_on: Option<&str>) -> PyResult<bool> {
pub fn has_diffs(file_a: &str, file_b: &str, ignore_blank_lines: bool) -> PyResult<bool> {
    let mut differ = ASCIIDiffer::new(Path::new(file_a), Path::new(file_b));
    differ.ignore_blank_lines = ignore_blank_lines;
    Ok(differ.has_diffs()?)
}

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.utils.differ", |m| {
        m.add_function(wrap_pyfunction!(has_diffs, m)?)?;
        Ok(())
    })
}
