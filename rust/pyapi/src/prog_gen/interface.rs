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
    #[allow(unused_variables)]
    fn add_test(
        &self,
        test_obj: &PyAny,
        if_failed: Option<&PyAny>,
        test_text: Option<String>,
    ) -> PyResult<()> {
        if let Ok(t) = test_obj.extract::<TestInvocation>() {
            PROG.add_test(t.name()?, Some(t.tester), t.test_id, Some(t.id))?;
        } else if let Ok(t) = test_obj.extract::<Test>() {
            PROG.add_test(t.name, Some(t.tester), Some(t.id), None)?;
        } else if let Ok(t) = test_obj.extract::<String>() {
            PROG.add_test(t, None, None, None)?;
        } else {
            return Err(TypeError::py_err(format!(
                "add_test must be given a valid test object, or a String, this is neither: {:?}",
                test_obj
            )));
        }
        Ok(())
    }

    /// Bin out
    #[allow(unused_variables)]
    fn bin(
        &self,
        number: usize,
        description: Option<String>,
        softbin: Option<usize>,
    ) -> PyResult<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn pass_bin(
        &self,
        number: usize,
        description: Option<String>,
        softbin: Option<usize>,
    ) -> PyResult<()> {
        Ok(())
    }
}
