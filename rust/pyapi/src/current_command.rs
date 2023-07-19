use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use super::extensions::Extensions;

pub const ATTR_NAME: &str = "_current_command_";

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "current_command")?;
    subm.add_wrapped(wrap_pyfunction!(get_command))?;
    subm.add_wrapped(wrap_pyfunction!(set_command))?;
    // subm.add_wrapped(wrap_pyfunction!(clear_command))?; FEATURE CLI Clearing Current Command
    subm.add_class::<CurrentCommand>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
pub fn get_command(py: Python) -> PyResult<PyRef<CurrentCommand>> {
    _origen!(py).getattr(ATTR_NAME)?.extract::<PyRef<CurrentCommand>>()
}

#[pyfunction]
fn set_command(
    py: Python,
    base_cmd: String,
    subcmds: Vec<String>,
    args: Py<PyDict>,
    ext_args: Py<PyDict>,
    arg_indices: Py<PyDict>,
    ext_arg_indices: Py<PyDict>,
    exts: &PyList
) -> PyResult<()> {
    let cmd = CurrentCommand {
        base: base_cmd,
        subcmds: subcmds,
        args: args,
        arg_indices: arg_indices,
        exts: Py::new(py, Extensions::new(py, exts, ext_args, ext_arg_indices)?)?,
    };
    _origen!(py).setattr(ATTR_NAME, Py::new(py, cmd)?)
}

// FEATURE CLI Clearing Current Command
// #[pyfunction]
// fn clear_command() -> PyResult<()> {
//     todo!()
// }

#[pyclass]
pub struct CurrentCommand {
    base: String,
    subcmds: Vec<String>,
    args: Py<PyDict>,
    arg_indices: Py<PyDict>,
    exts: Py<Extensions>,
}

#[pymethods]
impl CurrentCommand {
    #[getter]
    pub fn cmd(&self) -> PyResult<String> {
        Ok(if self.subcmds.is_empty() {
            self.base.to_string()
        } else {
            format!("{}.{}", self.base, self.subcmds.join("."))
        })
    }

    #[getter]
    pub fn base_cmd(&self) -> PyResult<&str> {
        Ok(&self.base)
    }

    #[getter]
    pub fn subcmds(&self) -> PyResult<Vec<String>> {
        Ok(self.subcmds.clone())
    }

    #[getter]
    pub fn exts(&self) -> PyResult<&Py<Extensions>> {
        Ok(&self.exts)
    }

    #[getter]
    pub fn args<'py>(&'py self, py: Python<'py>) -> PyResult<&'py PyDict> {
        Ok(self.args.as_ref(py))
    }

    #[getter]
    pub fn arg_indices<'py>(&'py self, py: Python<'py>) -> PyResult<&'py PyDict> {
        Ok(self.arg_indices.as_ref(py))
    }
}