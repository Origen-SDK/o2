use super::producer::exec_file;
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
    fn new(obj: &PyRawObject) {
        obj.init(PyInterface {});
    }

    fn include(&self, path: &str) -> PyResult<()> {
        let file = origen::with_current_job(|job| {
            job.resolve_file_reference(Path::new(path), Some(vec!["py"]))
        })?;
        log_trace!("Found include file '{}'", file.display());
        let job_id = origen::with_current_job(|job| Ok(job.id))?;
        exec_file(&file, job_id)?;
        Ok(())
    }

    /// Add a test to the flow
    fn test(&self, name: &str) -> PyResult<()> {
        Ok(())
    }
}
