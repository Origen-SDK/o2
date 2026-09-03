use crate::py_submodule;
use origen_metal::utils::differ::{ASCIIDiffer, Differ};
use pyo3::prelude::*;
use std::path::Path;

#[pyfunction]
#[pyo3(signature=(file_a, file_b, ignore_comments = None, ignore_blocks = None, ignore_blank_lines = true))]
pub fn has_diffs(
    file_a: &str,
    file_b: &str,
    ignore_comments: Option<Vec<String>>,
    ignore_blocks: Option<Vec<Vec<String>>>,
    ignore_blank_lines: bool,
) -> PyResult<bool> {
    let mut differ = ASCIIDiffer::new(Path::new(file_a), Path::new(file_b));
    differ.ignore_blank_lines = ignore_blank_lines;
    if let Some(chars) = ignore_comments {
        for c in chars {
            differ.ignore_comments(&c)?;
        }
    }
    if let Some(blocks) = ignore_blocks {
        for b in blocks {
            differ.ignore_block(&b[0], &b[1])?;
        }
    }
    Ok(differ.has_diffs()?)
}

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    py_submodule(py, parent, "origen_metal.utils.differ", |m| {
        m.add_function(wrap_pyfunction!(has_diffs, m)?)?;
        Ok(())
    })
}
