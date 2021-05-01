use pyo3::prelude::*;
use crate::application::{get_pyapp, PyApplication};
use super::RunResult;
use crate::runtime_error;

pub struct UnitTester {}

impl origen::core::frontend::UnitTester for UnitTester {
    fn run(&self) -> origen::Result<origen::core::frontend::UnitTestStatus> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let ut = PyApplication::_get_ut(pyapp, py)?;
        let pystat = ut.call_method0(py, "run")?;
        let stat = pystat.extract::<PyRef<RunResult>>(py)?;
        match stat.orr.as_ref() {
            Some(rr) => Ok(rr.clone()),
            None => runtime_error!("Incomplete or Uninitialized RunResult encountered")?
        }
    }
}
