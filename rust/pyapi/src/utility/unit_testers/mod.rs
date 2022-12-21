pub mod _frontend;

use pyapi_metal::_helpers::map_to_pydict;
use crate::runtime_error;
use origen::STATUS;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use origen_metal::{Outcome, OutcomeSubtypes};

#[pymodule]
pub fn unit_testers(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RunResult>()?;
    m.add_wrapped(wrap_pyfunction!(app_unit_tester))?;
    Ok(())
}

// TODO see if this is needed and move to OM, or try to use Outcome
#[pyclass(subclass)]
pub struct RunResult {
    // Origen Run Result
    pub orr: Option<Outcome>,
}

#[pymethods]
impl RunResult {
    #[classmethod]
    fn __init__(
        _cls: &PyType,
        instance: &PyAny,
        passed: bool,
        output: Option<String>,
    ) -> PyResult<()> {
        let mut i = instance.extract::<PyRefMut<Self>>()?;
        i.orr = Some({
            let mut o = Outcome::new_pass_or_fail(passed);
            if let Some(out) = output {
                o.set_msg(out);
            }
            o.subtype = Some(OutcomeSubtypes::UnitTestStatus);
            o
        });
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { orr: None }
    }

    #[getter]
    fn passed(&self) -> PyResult<bool> {
        Ok(self.get_orr()?.passed())
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(!self.passed()?)
    }
}

impl RunResult {
    fn get_orr(&self) -> PyResult<&Outcome> {
        match self.orr.as_ref() {
            Some(r) => Ok(r),
            None => runtime_error!("UnitTest Result has not been fully initialized yet!"),
        }
    }
}

// Returns a new pytest driver with the parameters from the app config
pub fn new_pytest_driver(py_config: &PyDict) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item("py_config", py_config)?;
    locals.set_item(
        "origen_pytester",
        py.import("origen.utility.unit_testers.pytest")?
            .to_object(py),
    )?;
    let pytester = py.eval("origen_pytester.PyTest(py_config)", Some(locals), None)?;
    Ok(pytester.to_object(py))
}

/// Creates a unit test driver from the application's ``config.toml``
#[pyfunction]
fn app_unit_tester() -> PyResult<Option<PyObject>> {
    // Raise an error if we aren't in an application instance
    let app;
    match &STATUS.app {
        Some(a) => app = a,
        None => {
            return runtime_error!(
                "Cannot retrieve the application's unit test config: no application found!"
            )
        }
    }

    let config = app.config();
    let gil = Python::acquire_gil();
    let py = gil.python();
    if let Some(ut_config) = &config.unit_tester {
        if let Some(c) = ut_config.get("system") {
            // Get the module and try to import it
            let split = c.rsplitn(1, ".");
            if split.count() == 2 {
                // Have a class (hopefully) of the form 'a.b.Class'
                // let py_config = hashmap_to_pydict(py, ut_config)?;
                // return runtime_error!("custom unit tester not supported yet!");
                todo!();
            } else {
                // fall back to some enumerated systems
                if &c.to_lowercase() == "pytest" {
                    let py_config = map_to_pydict(py, &mut ut_config.iter())?;
                    Ok(Some(new_pytest_driver(py_config.as_ref(py))?))
                } else if &c.to_lowercase() == "none" {
                    Ok(None)
                } else {
                    return runtime_error!(format!("Unrecognized unit tester system '{}'", c));
                }
            }
        } else {
            // Invalid config
            return runtime_error!("Could not discern unit tester from app config");
        }
    } else {
        let temp = HashMap::<String, String>::new();
        let py_config = map_to_pydict(py, &mut temp.iter())?;
        Ok(Some(new_pytest_driver(py_config.as_ref(py))?))
    }
}
