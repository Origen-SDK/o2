use crate::application::{get_pyapp, PyApplication};
use origen::Result as OResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::path::Path;

pub struct RC {}

impl origen::core::frontend::RC for RC {
    fn is_modified(&self) -> origen::Result<bool> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rc = PyApplication::_get_rc(pyapp, py)?;
        let stat = rc.call_method0(py, "status")?;
        let r = stat.getattr(py, "is_modified")?;
        Ok(r.extract::<bool>(py)?)
    }

    fn status(&self) -> OResult<origen::revision_control::Status> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rc = PyApplication::_get_rc(pyapp, py)?;
        let stat = rc.call_method0(py, "status")?;
        let rusty_stat = stat.extract::<PyRef<crate::utility::revision_control::Status>>(py)?;
        Ok(rusty_stat.stat().clone())
    }

    fn checkin(&self, files_or_dirs: Option<Vec<&Path>>, msg: &str) -> OResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rc = PyApplication::_get_rc(pyapp, py)?;

        let kwargs = PyDict::new(py);
        kwargs.set_item("msg", msg)?;

        let r;
        if let Some(f) = files_or_dirs {
            let pyfiles = PyTuple::new(
                py,
                f.iter()
                    .map(|fd| fd.display().to_string())
                    .collect::<Vec<String>>(),
            );
            r = rc.call_method(py, "checkin", pyfiles, Some(kwargs))?;
        } else {
            r = rc.call_method(
                py,
                "checkin_all",
                PyTuple::new(py, Vec::<u8>::new()),
                Some(kwargs),
            )?;
        }
        Ok(r.extract::<String>(py)?)
    }

    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> OResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rc = PyApplication::_get_rc(pyapp, py)?;

        let kwargs = PyDict::new(py);
        kwargs.set_item("msg", msg)?;
        kwargs.set_item("force", force)?;

        rc.call_method(py, "tag", PyTuple::new(py, &[tag]), Some(kwargs))?;
        Ok(())
    }
}
