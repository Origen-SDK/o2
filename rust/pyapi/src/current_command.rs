use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use super::extensions::Extensions;
use std::path::PathBuf;
use std::fmt;

pub const ATTR_NAME: &str = "_current_command_";

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "current_command")?;
    subm.add_wrapped(wrap_pyfunction!(get_command))?;
    subm.add_wrapped(wrap_pyfunction!(set_command))?;
    // subm.add_wrapped(wrap_pyfunction!(clear_command))?; FEATURE CLI Clearing Current Command
    subm.add_class::<CurrentCommand>()?;
    subm.add_class::<SourceType>()?;
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
    exts: &PyList,
    source_type: SourceType,
    source_path: Option<PathBuf>,
    source_plugin: Option<String>,
) -> PyResult<()> {
    let cmd = CurrentCommand {
        base: base_cmd,
        subcmds: subcmds,
        args: args,
        arg_indices: arg_indices,
        exts: Py::new(py, Extensions::new(py, exts, ext_args, ext_arg_indices)?)?,
        source: Py::new(py, CommandSource::new(source_type, source_path, source_plugin))?,
    };
    _origen!(py).setattr(ATTR_NAME, Py::new(py, cmd)?)
}

#[pyclass]
#[derive(Clone)]
pub enum SourceType {
    Core, Plugin, Aux, App
}

#[pymethods]
impl SourceType {
    #[getter]
    fn is_core_cmd(&self) -> bool {
        matches!(self, Self::Core)
    }

    #[getter]
    fn is_plugin_cmd(&self) -> bool {
        matches!(self, Self::Plugin)
    }

    #[getter]
    fn is_aux_cmd(&self) -> bool {
        matches!(self, Self::Aux)
    }

    #[getter]
    fn is_app_cmd(&self) -> bool {
        matches!(self, Self::App)
    }

    fn __str__(&self) -> &str {
        match self {
            Self::Core => "core",
            Self::Plugin => "plugin",
            Self::Aux => "aux",
            Self::App => "app",
        }
    }

    #[getter]
    fn root_name(&self) -> &str {
        match self {
            Self::Core => "core",
            Self::Plugin => "plugin",
            Self::Aux => "aux_ns",
            Self::App => "app",
        }
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source_type_str = match self {
            SourceType::Core => "Core",
            SourceType::Plugin => "Plugin",
            SourceType::Aux => "Aux",
            SourceType::App => "App",
        };
        write!(f, "{}", source_type_str)
    }
}

#[pyclass]
pub struct CommandSource {
    source_type: SourceType,
    path: Option<PathBuf>,
    plugin: Option<String>, // Store plugin name and getter will return actual plugin
}

impl CommandSource {
    pub fn new(source_type: SourceType, path: Option<PathBuf>, plugin: Option<String>) -> Self {
        Self {
            source_type: source_type,
            path: path,
            plugin: plugin
        }
    }
}

#[pymethods]
impl CommandSource {
    #[getter]
    fn source_type(&self) -> PyResult<SourceType> {
        Ok(self.source_type.clone())
    }

    #[getter]
    fn path(&self) -> PyResult<Option<&PathBuf>> {
        Ok(self.path.as_ref())
    }

    #[getter]
    fn plugin<'py>(&'py self, py: Python<'py>) -> PyResult<Option<&'py PyAny>> {
        if let Some(pl) = self.plugin.as_ref() {
            Ok(Some(get_plugin!(py, pl)))
        } else {
            Ok(None)
        }
    }
}

// FEATURE CLI Clearing Current Command
// #[pyfunction]
// fn clear_command() -> PyResult<()> {
//     todo!()
// }

#[pyclass]
#[derive(Debug)]
pub struct CurrentCommand {
    base: String,
    subcmds: Vec<String>,
    args: Py<PyDict>,
    arg_indices: Py<PyDict>,
    exts: Py<Extensions>,
    source: Py<CommandSource>,
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

    #[getter]
    pub fn command_source<'py>(&'py self, _py: Python<'py>) -> PyResult<&Py<CommandSource>> {
        Ok(&self.source)
    }

    #[getter]
    pub fn source<'py>(&'py self, py: Python<'py>) -> PyResult<&Py<CommandSource>> {
        self.command_source(py)
    }

    pub fn __str__<'py>(&'py self, py: Python<'py>) -> PyResult<String> {
        let src_type = self.source.as_ref(py).borrow().source_type()?;
        let mut s = format!(
            "Showing Current Command:\n \
            base command: {}\n \
            sub-commands: {}\n \
            args:         {}\n \
            arg-indices:  {}\n \
            extensions:   {}\n \
            source-type:  {}",
            self.base,
            self.subcmds.join(", "),
            self.args.as_ref(py).str()?,
            self.arg_indices.as_ref(py).str()?,
            self.exts.as_ref(py).call_method0("keys")?.str()?,
            &src_type,
        );
        match src_type {
            SourceType::Core => {},
            SourceType::App | SourceType::Aux | SourceType::Plugin => {
                s += &format!("\n source-path: {}", self.source.as_ref(py).borrow().path.as_ref().unwrap().display())
            }
        }
        match src_type {
            SourceType::Core | SourceType::App => {},
            SourceType::Aux | SourceType::Plugin => {
                s += &format!("\n source-plugin: {}", self.source.as_ref(py).borrow().plugin.as_ref().unwrap())
            }
        }
        Ok(s)
    }
}