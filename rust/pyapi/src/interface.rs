use pyo3::prelude::*;
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
    fn test(&self, _name: &str) -> PyResult<()> {
        Ok(())
    }
}
