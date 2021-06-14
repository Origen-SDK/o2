use crate::application::{get_pyapp, PyApplication};
use origen::Result as OResult;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use origen::core::frontend as ofrontend;
use ofrontend::BuildResult as OrigenBuildResult;
use ofrontend::UploadResult as OrigenUploadResult;
use crate::utility::results::{BuildResult, UploadResult};

pub struct Publisher {}

impl ofrontend::Publisher for Publisher {
    fn build_package(&self) -> OResult<OrigenBuildResult> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let pb = PyApplication::_get_publisher(pyapp, py)?;
        let py_pbr = pb.call_method0(py, "build_package")?;
        let pbr = py_pbr.extract::<PyRef<BuildResult>>(py)?;
        Ok(pbr.build_result()?.clone())
    }

    fn upload(&self, build_result: &OrigenBuildResult) -> OResult<OrigenUploadResult> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let pb = PyApplication::_get_publisher(pyapp, py)?;
        let py_pbr = pb.call_method1(py, "upload", PyTuple::new(py, &[BuildResult::to_py(py, build_result)?]))?;
        let pur = py_pbr.extract::<PyRef<UploadResult>>(py)?;
        Ok(pur.upload_result()?.clone())
    }
}