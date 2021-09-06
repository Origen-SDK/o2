use origen_metal::Outcome as OrigenOutcome;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};

#[macro_export]
macro_rules! partially_initialized_outcome_error {
    ($outcome_type:expr) => {{
        crate::bail_with_runtime_error!(format!(
            "Partially-initialized {} encountered",
            $outcome_type
        ))?
    }};
}

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "outcomes")?;
    subm.add_class::<Outcome>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass(subclass)]
pub struct Outcome {
    pub origen_outcome: Option<OrigenOutcome>,
}

#[pymethods]
impl Outcome {
    /// Provide an __init__ for typical Python subclass initialization.
    /// Otherwise, ``new`` will initialize.
    #[classmethod]
    #[args(message = "None", metadata = "None", use_pass_fail = "false")]
    fn __init__(
        _cls: &PyType,
        mut instance: PyRefMut<Self>,
        succeeded: bool,
        message: Option<String>,
        use_pass_fail: bool,
        metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        instance.init(succeeded, message, use_pass_fail, metadata)?;
        Ok(())
    }

    #[new]
    #[args(message = "None", metadata = "None", use_pass_fail = "false")]
    fn new(
        succeeded: bool,
        message: Option<String>,
        use_pass_fail: bool,
        metadata: Option<&PyDict>,
    ) -> PyResult<Self> {
        let mut obj = Self {
            origen_outcome: None,
        };
        obj.init(succeeded, message, use_pass_fail, metadata)?;
        Ok(obj)
    }

    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.origen_outcome()?.succeeded())
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(!self.succeeded()?)
    }

    #[getter]
    fn message(&self) -> PyResult<Option<String>> {
        Ok(self.origen_outcome()?.message.clone())
    }

    // #[getter]
    // fn metadata(&self) -> PyResult<PyObject> {
    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     into_optional_pyobj(py, self.origen_outcome()?.metadata.as_ref())
    // }

    pub fn gist(&self) -> PyResult<()> {
        Ok(self.origen_outcome()?.gist())
    }

    pub fn summarize_and_exit(&self) -> PyResult<()> {
        Ok(self.origen_outcome()?.summarize_and_exit())
    }
}

impl Outcome {
    pub fn init(
        &mut self,
        succeeded: bool,
        message: Option<String>,
        use_pass_fail: bool,
        _metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        let mut outcome;
        if use_pass_fail {
            outcome = OrigenOutcome::new_pass_or_fail(succeeded);
        } else {
            outcome = OrigenOutcome::new_success_or_fail(succeeded);
        }
        outcome.message = message;
        // outcome.metadata = from_optional_pydict(metadata)?;
        self.origen_outcome = Some(outcome);
        Ok(())
    }

    pub fn origen_outcome(&self) -> PyResult<&OrigenOutcome> {
        match self.origen_outcome.as_ref() {
            Some(r) => Ok(r),
            None => return crate::partially_initialized_outcome_error!("Outcome"),
        }
    }

    pub fn into_origen(&self) -> PyResult<OrigenOutcome> {
        Ok(self.origen_outcome()?.clone())
    }

    pub fn to_py(py: Python, origen_outcome: &OrigenOutcome) -> PyResult<Py<Self>> {
        Py::new(
            py,
            Self {
                origen_outcome: Some(origen_outcome.clone()),
            },
        )
    }

    pub fn from_origen(origen_outcome: OrigenOutcome) -> Self {
        Self {
            origen_outcome: Some(origen_outcome),
        }
    }
}
