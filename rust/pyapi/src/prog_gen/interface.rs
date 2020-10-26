use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::Path;
use crate::tester_apis::to_test_invocation;

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
    fn new(obj: &PyRawObject) {
        obj.init(PyInterface {});
    }

    fn resolve_file_reference(&self, path: &str) -> PyResult<String> {
        let file = origen::with_current_job(|job| {
            job.resolve_file_reference(Path::new(path), Some(vec!["py"]))
        })?;
        Ok(file.to_str().unwrap().to_string())
    }

    /// Add a test to the flow
    fn add_test(&self, test_obj: &PyAny) -> PyResult<()> {
        if let Some(t) = to_test_invocation(&test_obj) {
            //log_info!("GAot V93K test! - {}", t.name);
            Ok(())
        } else {
            log_error!("Could not convert: {:?}", test_obj);
            Ok(())
        }
    }

    /// Bin out
    fn bin(&self, _number: usize) -> PyResult<()> {
        Ok(())
    }
}
