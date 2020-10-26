use crate::prog_gen::{Test, TestInvocation};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::Path;

#[pymodule]
pub fn interface(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInterface>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyInterface {
    //python_testers: HashMap<String, PyObject>,
//instantiated_testers: HashMap<String, PyObject>,
//metadata: Vec<PyObject>,
}

#[pymethods]
impl PyInterface {
    #[new]
    fn new() -> Self {
        PyInterface {}
    }

    fn resolve_file_reference(&self, path: &str) -> PyResult<String> {
        let file = origen::with_current_job(|job| {
            job.resolve_file_reference(Path::new(path), Some(vec!["py"]))
        })?;
        Ok(file.to_str().unwrap().to_string())
    }

    /// Add a test to the flow
    fn add_test(&self, test_obj: &PyAny) -> PyResult<()> {
        if let Ok(t) = test_obj.extract::<TestInvocation>() {
            log_info!("Got a test invocation!");
        } else if let Ok(t) = test_obj.extract::<Test>() {
            log_info!("Got a testvocation!");
        } else {
            log_error!("Could not convert: {:?}", test_obj);
        }
        Ok(())
    }

    /// Bin out
    fn bin(&self, _number: usize) -> PyResult<()> {
        Ok(())
    }
}
