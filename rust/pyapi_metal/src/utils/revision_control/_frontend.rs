use crate::framework::Outcome as PyOutcome;
use origen_metal::frontend::RevisionControlFrontendAPI;
use origen_metal::utils::revision_control::Status;
use origen_metal::Outcome as OMOutcome;
use origen_metal::Result as OMResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::path::Path;
use super::Status as PyStatus;

pub struct RevisionControlFrontend {}

impl RevisionControlFrontendAPI for RevisionControlFrontend {
    fn is_modified(&self) -> OMResult<bool> {
        let stat = self.status()?;
        Ok(stat.is_modified())
    }

    fn status(&self) -> OMResult<Status> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            let stat = rc.call_method0(py, "status")?;
            let rusty_stat = stat.extract::<PyRef<PyStatus>>(py)?;
            rusty_stat.into_origen()
        })?;
        Ok(r)
    }

    fn checkin(&self, files_or_dirs: Option<Vec<&Path>>, msg: &str, dry_run: bool) -> OMResult<OMOutcome> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("msg", msg)?;
            kwargs.set_item("dry_run", dry_run)?;

            let res;
            if let Some(f) = files_or_dirs.as_ref() {
                let pyfiles = PyTuple::new(
                    py,
                    f.iter()
                        .map(|fd| fd.display().to_string())
                        .collect::<Vec<String>>(),
                );
                res = rc.call_method(py, "checkin", pyfiles, Some(kwargs))?;
            } else {
                res = rc.call_method(
                    py,
                    "checkin_all",
                    PyTuple::new(py, Vec::<u8>::new()),
                    Some(kwargs),
                )?;
            }
            let outcome = res.extract::<PyRef<PyOutcome>>(py)?;
            outcome.into_origen()
        });
        Ok(r?)
    }

    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> OMResult<()> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("msg", msg)?;
            kwargs.set_item("force", force)?;
    
            rc.call_method(py, "tag", PyTuple::new(py, &[tag]), Some(kwargs))?;
            Ok(())
        });
        Ok(r?)
    }

    fn system(&self) -> OMResult<String> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            rc.call_method0(py, "system")?.extract::<String>(py)
        });
        Ok(r?)
    }

    fn init(&self) -> OMResult<OMOutcome> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            let outcome = rc.call_method0(py, "init")?;
            let rusty_outcome = outcome.extract::<PyRef<PyOutcome>>(py)?;
            rusty_outcome.into_origen()
        });
        Ok(r?)
    }
}
