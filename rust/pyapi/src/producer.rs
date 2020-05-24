use origen::STATUS;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::path::Path;

#[pymodule]
pub fn producer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyProducer>()?;
    m.add_class::<PyPattern>()?;
    m.add_class::<PyJob>()?;

    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyJob {
    id: usize,
}

#[pymethods]
impl PyJob {
    /// Execute the job's file. There is currently no checking that this is called on a job
    /// which has a file component and it will panic if not.
    pub fn run(&self) -> PyResult<()> {
        let file;
        {
            let p = origen::producer();
            let j = &p.jobs[self.id];
            file = j.source_file().unwrap().to_path_buf();
        }
        self.exec_file(&file)
    }

    #[getter]
    pub fn id(&self) -> PyResult<usize> {
        Ok(self.id)
    }

    #[getter]
    pub fn source_file(&self) -> PyResult<String> {
        let p = origen::producer();
        let j = &p.jobs[self.id];
        Ok(j.source_file().unwrap().to_str().unwrap().to_string())
    }
}

impl PyJob {
    /// Executes a pattern or flow source file
    pub fn exec_file(&self, path: &Path) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = [("origen", py.import("origen")?)].into_py_dict(py);
        py.eval(
            &format!(
                "origen.load_file(r\"{}\", locals={{**origen.standard_context(), **{{ \
                    'Pattern': lambda **kwargs : __import__(\"origen\").producer.Pattern(__import__(\"origen\").producer.get_job_by_id({}), **kwargs), \
                    'Flow': lambda **kwargs : __import__(\"origen\").producer.Flow(__import__(\"origen\").producer.get_job_by_id({}), **kwargs) \
                 }}}})",
                path.display(),
                self.id,
                self.id
            ),
            None,
            Some(locals)
        )?;
        Ok(())
    }
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyProducer {}

#[pymethods]
impl PyProducer {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(PyProducer {});
    }

    fn create_job(&self, command: &str, file: Option<&str>) -> PyResult<Py<PyJob>> {
        let mut p = origen::producer();
        let j = match file {
            None => p.create_job(command, None)?,
            Some(f) => p.create_job(command, Some(Path::new(f)))?,
        };

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PyJob { id: j.id }).unwrap())
    }

    fn get_job_by_id(&self, id: usize) -> PyResult<Py<PyJob>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PyJob { id: id }).unwrap())
    }

    // Hard-coded for now
    fn output_dir(&self) -> PyResult<String> {
        Ok(format!("{}", STATUS.root.display()))
    }
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyPattern {
    // ...
}

#[pymethods]
impl PyPattern {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(PyPattern {});
    }
}
