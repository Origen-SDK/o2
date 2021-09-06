use crate::framework::Outcome;
use origen_metal::frontend::RevisionControlFrontendAPI;
use origen_metal::utils::revision_control::Status;
use origen_metal::Outcome as OMOutcome;
use origen_metal::Result as OMResult;
use pyo3::prelude::*;
use std::path::Path;

pub struct RevisionControlFrontend {}

impl RevisionControlFrontendAPI for RevisionControlFrontend {
    fn is_modified(&self) -> OMResult<bool> {
        todo!()
    }

    fn status(&self) -> OMResult<Status> {
        todo!()
    }

    fn checkin(&self, _: Option<Vec<&Path>>, _: &str, _: bool) -> OMResult<OMOutcome> {
        todo!()
    }

    fn tag(&self, _: &str, _: bool, _: Option<&str>) -> OMResult<()> {
        todo!()
    }

    fn system(&self) -> OMResult<String> {
        todo!()
    }

    fn init(&self) -> OMResult<OMOutcome> {
        let r = crate::frontend::with_required_rc(|py, rc| {
            let outcome = rc.call_method0(py, "init")?;
            let rusty_outcome = outcome.extract::<PyRef<Outcome>>(py)?;
            rusty_outcome.into_origen()
        });
        Ok(r?)
    }
}
