use origen::Error;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
//use std::collections::HashMap;
use std::path::PathBuf;

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
        //let cmd;
        //{
        //    let p = origen::producer();
        //    let j = &p.jobs[self.id];
        //    cmd = j.command.clone();
        //}

        //let gil = Python::acquire_gil();
        //let py = gil.python();
        //let locals = [("origen", py.import("origen")?)].into_py_dict(py);
        //py.eval(
        //    &format!(
        //        "origen.load_file(r\"{}\", locals={{**origen.standard_context(), **{{ \
        //            'Pattern': lambda **kwargs : __import__(\"origen\").producer.Pattern(__import__(\"origen\").producer.get_job_by_id({}), **kwargs), \
        //            'Flow': lambda **kwargs : __import__(\"origen\").producer.Flow(__import__(\"origen\").producer.get_job_by_id({}), **kwargs) \
        //         }}}})",
        //        &cmd,
        //        self.id,
        //        self.id
        //    ),
        //    None,
        //    Some(locals)
        //)?;
        Ok(())
    }
}
