use crate::prog_gen::{Test, TestInvocation};
use origen::PROG;
use pyo3::exceptions::TypeError;
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
            PROG.add_test(Some(t.tester), t.test_id, Some(t.id), None)?;
        } else if let Ok(t) = test_obj.extract::<Test>() {
            PROG.add_test(Some(t.tester), Some(t.id), None, None)?;
        } else if let Ok(t) = test_obj.extract::<String>() {
            PROG.add_test(None, None, None, Some(t))?;
        } else {
            return Err(TypeError::py_err(format!(
                "add_test must be given a valid test object, or a String, this is neither: {:?}",
                test_obj
            )));
        }
        Ok(())
    }

    /// Bin out
    fn bin(&self, _number: usize) -> PyResult<()> {
        Ok(())
    }
}
