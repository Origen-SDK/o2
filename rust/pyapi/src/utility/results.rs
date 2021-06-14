use crate::utility::metadata::{from_optional_pydict, into_optional_pyobj};
use pyo3::prelude::*;
use origen::core::frontend::BuildResult as OrigenBuildResult;
use origen::core::frontend::UploadResult as OrigenUploadResult;
use origen::utility::command_helpers::ExecResult as OrigenExecResult;
use pyo3::types::{PyDict, PyType};

#[macro_export]
macro_rules! incomplete_result_error {
    ($result_type:expr) => {{
        crate::runtime_error!(format!("Incomplete or Uninitialized {} encountered", $result_type))?
    }};
}

#[pymodule]
pub fn results(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BuildResult>()?;
    m.add_class::<UploadResult>()?;
    m.add_class::<ExecResult>()?;
    Ok(())
}

/// Generic build result
#[pyclass(subclass)]
pub struct BuildResult {
    // Origen Build Result
    pub build_result: Option<OrigenBuildResult>,
}

#[pymethods]
impl BuildResult {
    #[classmethod]
    #[args(build_contents="None", message="None",  metadata="None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        succeeded: bool,
        build_contents: Option<Vec<String>>,
        message: Option<String>,
        metadata: Option<&PyDict>
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        i.build_result = Some(OrigenBuildResult {
            succeeded: succeeded,
            build_contents: build_contents,
            message: message,
            metadata: from_optional_pydict(metadata)?
        });
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { build_result: None }
    }

    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.build_result()?.succeeded)
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(!self.succeeded()?)
    }

    #[getter]
    fn build_contents(&self) -> PyResult<Option<Vec<String>>> {
        Ok(self.build_result()?.build_contents.clone())
    }

    #[getter]
    fn message(&self) -> PyResult<Option<String>> {
        Ok(self.build_result()?.message.clone())
    }

    #[getter]
    fn metadata(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        into_optional_pyobj(py, self.build_result()?.metadata.as_ref())
    }
}

impl BuildResult {
    pub fn build_result(&self) -> PyResult<&OrigenBuildResult> {
        match self.build_result.as_ref() {
            Some(r) => Ok(r),
            None => {
                return crate::incomplete_result_error!("Build Result")
            }
        }
    }

    pub fn to_py(py: Python, build_result: &OrigenBuildResult) -> PyResult<Py<Self>> {
        Py::new(py, Self {
            build_result: Some(build_result.clone())
        })
    }
}

/// Generic upload result
#[pyclass(subclass)]
pub struct UploadResult {
    // Origen Upload Result
    pub upload_result: Option<origen::core::frontend::UploadResult>,
}

#[pymethods]
impl UploadResult {
    #[classmethod]
    #[args(message="None",  metadata="None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        succeeded: bool,
        message: Option<String>,
        metadata: Option<&PyDict>
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        i.upload_result = Some(OrigenUploadResult {
            succeeded: succeeded,
            message: message,
            metadata: from_optional_pydict(metadata)?
        });
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { upload_result: None }
    }
}

impl UploadResult {
    pub fn upload_result(&self) -> PyResult<&OrigenUploadResult> {
        match self.upload_result.as_ref() {
            Some(r) => Ok(r),
            None => {
                crate::incomplete_result_error!("Upload Result")
            }
        }
    }
}

#[pyclass]
pub struct ExecResult {
    pub exec_result: Option<OrigenExecResult>
}

#[pymethods]
impl ExecResult {
    #[classmethod]
    #[args(stdout="None",  stderr="None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        exit_code: i32,
        stdout: Option<Vec<String>>,
        stderr: Option<Vec<String>>,
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        i.exec_result = Some(OrigenExecResult {
            exit_code: exit_code,
            stdout: stdout,
            stderr: stderr,
        });
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { exec_result: None }
    }

    #[getter]
    pub fn exit_code(&self) -> PyResult<i32> {
        Ok(self.exec_result()?.exit_code)
    }

    #[getter]
    pub fn stdout(&self) -> PyResult<Option<Vec<String>>> {
        Ok(self.exec_result()?.stdout.clone())
    }

    #[getter]
    pub fn stderr(&self) -> PyResult<Option<Vec<String>>> {
        Ok(self.exec_result()?.stderr.clone())
    }

    pub fn succeeded(&self) -> PyResult<bool> {
        Ok(self.exec_result()?.succeeded())
    }

    pub fn failed(&self) -> PyResult<bool> {
        Ok(self.exec_result()?.failed())
    }
}

impl ExecResult {
    fn exec_result(&self) -> PyResult<&OrigenExecResult> {
        match self.exec_result.as_ref() {
            Some(r) => Ok(r),
            None => crate::incomplete_result_error!("Exec Result")
        }
    }
}