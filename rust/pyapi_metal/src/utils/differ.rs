use crate::py_submodule;
use origen_metal::utils::differ::{ASCIIDiffer, Differ};
use pyo3::prelude::*;
use std::path::Path;

/// This function compares the given two files a
#[pyfunction(
    ignore_comments = "None",
    suspend_on = "None",
    resume_on = "None",
    ignore_blank_lines = "true"
)]
pub fn has_diffs(
    file_a: &str,
    file_b: &str,
    ignore_comments: Option<String>,
    suspend_on: Option<String>,
    resume_on: Option<String>,
    ignore_blank_lines: bool,
) -> PyResult<bool> {
    let mut differ = ASCIIDiffer::new(Path::new(file_a), Path::new(file_b));
    differ.ignore_blank_lines = ignore_blank_lines;
    if let Some(c) = ignore_comments {
        differ.ignore_comments(&c)?;
    }
    if let Some(c) = suspend_on {
        differ.suspend_on(&c)?;
    }
    if let Some(c) = resume_on {
        differ.resume_on(&c)?;
    }
    Ok(differ.has_diffs()?)
}

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.utils.differ", |m| {
        m.add_function(wrap_pyfunction!(has_diffs, m)?)?;
        Ok(())
    })
}
