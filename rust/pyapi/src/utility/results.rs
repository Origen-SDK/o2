use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyapi_metal::prelude::typed_value;
use pyapi_metal::runtime_error;
use origen_metal::{Outcome, OutcomeSubtypes, TypedValue, Result};

#[macro_export]
macro_rules! incomplete_result_error {
    ($result_type:expr) => {{
        crate::runtime_error!(format!(
            "Incomplete or Uninitialized {} encountered",
            $result_type
        ))?
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
    pub build_result: Option<Outcome>,
}

#[pymethods]
impl BuildResult {
    #[classmethod]
    #[args(build_contents = "None", message = "None", metadata = "None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        succeeded: bool,
        build_contents: Option<Vec<String>>,
        message: Option<String>,
        metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        let mut o = Outcome::new_success_or_fail(succeeded);
        o.subtype = Some(OutcomeSubtypes::BuildResult);
        o.message = message;
        o.metadata = typed_value::from_optional_pydict(metadata)?;
        o.insert_keyword_result("build_contents", build_contents);
        i.build_result = Some(o);
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { build_result: None }
    }

    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.build_result()?.succeeded())
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(!self.succeeded()?)
    }

    #[getter]
    fn build_contents(&self) -> PyResult<Option<Vec<String>>> {
        match self.build_result()?.require_keyword_result("build_contents")? {
            TypedValue::None => Ok(None),
            TypedValue::Vec(v) => Ok(Some(v.iter().map( |i| i.as_string()).collect::<Result<Vec<String>>>()?)),
            _ => runtime_error!("Cannot extract build contents as either 'None' or as a 'list of strs'")
        }
    }

    #[getter]
    fn message(&self) -> PyResult<Option<String>> {
        Ok(self.build_result()?.message.clone())
    }

    #[getter]
    fn metadata<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyDict>> {
        typed_value::into_optional_pydict(py, self.build_result()?.metadata.as_ref())
    }
}

impl BuildResult {
    pub fn build_result(&self) -> PyResult<&Outcome> {
        match self.build_result.as_ref() {
            Some(r) => Ok(r),
            None => return crate::incomplete_result_error!("Build Result"),
        }
    }

    pub fn to_py(py: Python, build_result: &Outcome) -> PyResult<Py<Self>> {
        Py::new(
            py,
            Self {
                build_result: Some(build_result.clone()),
            },
        )
    }
}

/// Generic upload result
#[pyclass(subclass)]
pub struct UploadResult {
    // Origen Upload Result
    pub upload_result: Option<Outcome>,
}

#[pymethods]
impl UploadResult {
    #[classmethod]
    #[args(message = "None", metadata = "None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        succeeded: bool,
        message: Option<String>,
        metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        let mut o = Outcome::new_success_or_fail(succeeded);
        o.subtype = Some(OutcomeSubtypes::UploadResult);
        o.message = message;
        o.metadata = typed_value::from_optional_pydict(metadata)?;
        i.upload_result = Some(o);
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self {
            upload_result: None,
        }
    }
}

impl UploadResult {
    pub fn upload_result(&self) -> PyResult<&Outcome> {
        match self.upload_result.as_ref() {
            Some(r) => Ok(r),
            None => crate::incomplete_result_error!("Upload Result"),
        }
    }
}

#[pyclass]
pub struct ExecResult {
    pub exec_result: Option<Outcome>,
}

#[pymethods]
impl ExecResult {
    #[classmethod]
    #[args(stdout = "None", stderr = "None")]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        exit_code: i32,
        stdout: Option<Vec<String>>,
        stderr: Option<Vec<String>>,
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        let mut o = Outcome::new_success_or_fail(exit_code == 0);
        o.subtype = Some(OutcomeSubtypes::ExecResult);
        o.insert_keyword_result("exit_code", exit_code);
        o.insert_keyword_result("stdout", stdout);
        o.insert_keyword_result("stderr", stderr);
        i.exec_result = Some(o);
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { exec_result: None }
    }

    #[getter]
    pub fn exit_code(&self) -> PyResult<i32> {
        Ok(self.exec_result()?.require_keyword_result("exit_code")?.try_into()?)
    }

    #[getter]
    pub fn stdout(&self) -> PyResult<Option<Vec<String>>> {
        Ok(match self.exec_result()?.require_keyword_result("stdout")?.as_option() {
            Some(out_lines) => Some(out_lines.try_into()?),
            None => None
        })
    }

    #[getter]
    pub fn stderr(&self) -> PyResult<Option<Vec<String>>> {
        Ok(match self.exec_result()?.require_keyword_result("stderr")?.as_option() {
            Some(err_lines) => Some(err_lines.try_into()?),
            None => None
        })
    }

    pub fn succeeded(&self) -> PyResult<bool> {
        Ok(self.exec_result()?.succeeded())
    }

    pub fn failed(&self) -> PyResult<bool> {
        Ok(self.exec_result()?.failed())
    }
}

impl ExecResult {
    fn exec_result(&self) -> PyResult<&Outcome> {
        match self.exec_result.as_ref() {
            Some(r) => Ok(r),
            None => crate::incomplete_result_error!("Exec Result"),
        }
    }
}
