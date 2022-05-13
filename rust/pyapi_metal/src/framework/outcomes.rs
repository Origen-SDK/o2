use crate::_helpers::typed_value::{
    from_optional_pydict, from_optional_pylist, into_optional_pydict, into_pytuple,
};
use origen_metal::Outcome as OrigenOutcome;
use origen_metal::TypedValueVec;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};

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

pub fn pyobj_into_om_outcome(py: Python, obj: PyObject) -> PyResult<OrigenOutcome> {
    if obj.is_none(py) {
        Outcome::new_om_inferred(None)
    } else if let Ok(o) = obj.extract::<PyRef<Outcome>>(py) {
        o.into_origen()
    } else if let Ok(s) = obj.extract::<String>(py) {
        let mut o = OrigenOutcome::new_succeeded();
        o.message = Some(s);
        Ok(o)
    } else {
        crate::type_error!(&format!(
            "Unable build Outcome out of Python type '{}'",
            obj.extract::<&PyAny>(py)?.get_type()
        ))
    }
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
        succeeded: &PyAny,
        message: Option<String>,
        positional_results: Option<&PyList>,
        keyword_results: Option<&PyDict>,
        use_pass_fail: bool,
        metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        instance.init(
            succeeded,
            message,
            positional_results,
            keyword_results,
            use_pass_fail,
            metadata,
        )?;
        Ok(())
    }

    #[new]
    #[args(message = "None", metadata = "None", use_pass_fail = "false")]
    fn new(
        succeeded: &PyAny,
        message: Option<String>,
        positional_results: Option<&PyList>,
        keyword_results: Option<&PyDict>,
        use_pass_fail: bool,
        metadata: Option<&PyDict>,
    ) -> PyResult<Self> {
        let mut obj = Self {
            origen_outcome: None,
        };
        obj.init(
            succeeded,
            message,
            positional_results,
            keyword_results,
            use_pass_fail,
            metadata,
        )?;
        Ok(obj)
    }

    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.origen_outcome()?.succeeded())
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(self.origen_outcome()?.failed())
    }

    #[getter]
    fn errored(&self) -> PyResult<bool> {
        Ok(self.origen_outcome()?.errored())
    }

    #[getter]
    fn message(&self) -> PyResult<Option<String>> {
        Ok(self.origen_outcome()?.message.clone())
    }

    #[getter]
    fn msg(&self) -> PyResult<Option<String>> {
        Ok(self.origen_outcome()?.msg().clone())
    }

    #[getter]
    fn inferred(&self) -> PyResult<Option<bool>> {
        Ok(self.origen_outcome()?.inferred)
    }

    #[getter]
    fn positional_results<'a>(&self, py: Python<'a>) -> PyResult<Option<&'a PyTuple>> {
        let retn;
        if let Some(pr) = self.origen_outcome()?.positional_results.as_ref() {
            retn = Some(into_pytuple(py, &mut pr.typed_values().iter())?);
        } else {
            retn = None;
        }
        Ok(retn)
    }

    #[getter]
    fn keyword_results<'a>(&self, py: Python<'a>) -> PyResult<Option<&'a PyDict>> {
        into_optional_pydict(py, self.origen_outcome()?.keyword_results.as_ref())
    }

    #[getter]
    fn metadata<'a>(&self, py: Python<'a>) -> PyResult<Option<&'a PyDict>> {
        into_optional_pydict(py, self.origen_outcome()?.metadata.as_ref())
    }

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
        succeeded: &PyAny,
        mut message: Option<String>,
        positional_results: Option<&PyList>,
        keyword_results: Option<&PyDict>,
        use_pass_fail: bool,
        metadata: Option<&PyDict>,
    ) -> PyResult<()> {
        let mut outcome;
        if let Ok(res) = succeeded.extract::<bool>() {
            if use_pass_fail {
                outcome = OrigenOutcome::new_pass_or_fail(res);
            } else {
                outcome = OrigenOutcome::new_success_or_fail(res);
            }
        } else if let Ok(err) = succeeded.downcast::<pyo3::exceptions::PyBaseException>() {
            outcome = OrigenOutcome::new_error();
            if message.is_none() {
                message = Some(err.to_string())
            }
        } else {
            return crate::type_error!(&format!(
                "Outcomes can only be build from a boolean value or an exception. Received {}",
                succeeded.get_type()
            ));
        }
        outcome.inferred = Some(false);
        outcome.message = message;
        outcome.positional_results = from_optional_pylist(positional_results)?;
        outcome.keyword_results = from_optional_pydict(keyword_results)?;
        outcome.metadata = from_optional_pydict(metadata)?;
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

    pub fn new_om_inferred(return_value: Option<&PyAny>) -> PyResult<OrigenOutcome> {
        let mut omo = OrigenOutcome::new_success();
        omo.inferred = Some(true);
        if let Some(rv) = return_value {
            let mut tvv = TypedValueVec::new();
            tvv.typed_values
                .push(crate::_helpers::typed_value::from_pyany(rv)?);
            omo.positional_results = Some(tvv);
        }
        Ok(omo)
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

    pub fn from_om(origen_outcome: OrigenOutcome) -> Self {
        Self::from_origen(origen_outcome)
    }
}

impl std::convert::From<OrigenOutcome> for Outcome {
    fn from(om: OrigenOutcome) -> Self {
        Self::from_origen(om)
    }
}

// TODO try to support this?
// impl std::convert::TryFrom<PyObject> for Outcome {
//     type Error = PyErr;

//     fn try_from(pyval: PyObject) -> PyResult<Self> {
//     }
// }
