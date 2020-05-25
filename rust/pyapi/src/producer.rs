use origen::STATUS;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

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
    #[getter]
    pub fn id(&self) -> PyResult<usize> {
        Ok(self.id)
    }

    #[getter]
    /// Returns the source file at the root of the job
    pub fn source_file(&self) -> PyResult<Option<String>> {
        Ok(origen::with_current_job(|job| {
            Ok(match job.source_file() {
                None => None,
                Some(f) => Some(format!("{}", f.display())),
            })
        })?)
    }

    #[getter]
    /// Returns the current file being executed by the job. This may be the same as the
    /// source_file or it could be different, for example if a flow has included a sub-flow file.
    pub fn current_file(&self) -> PyResult<Option<String>> {
        Ok(origen::with_current_job(|job| {
            Ok(match job.current_file() {
                None => None,
                Some(f) => Some(format!("{}", f.display())),
            })
        })?)
    }

    /// Add the given file to the job's files stack
    fn add_file(&self, file: String) -> PyResult<()> {
        Ok(origen::with_current_job_mut(|job| {
            job.files.push(PathBuf::from(file.clone()));
            Ok(())
        })?)
    }

    /// Pop a file off the job's files stack
    fn pop_file(&self) -> PyResult<()> {
        Ok(origen::with_current_job_mut(|job| {
            job.files.pop();
            Ok(())
        })?)
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

    fn create_job(&self, command: &str, file: Option<&str>) -> PyResult<PyJob> {
        let mut p = origen::producer();
        let j = match file {
            None => p.create_job(command, None)?,
            Some(f) => p.create_job(command, Some(Path::new(f)))?,
        };
        Ok(PyJob { id: j.id })
    }

    #[getter]
    fn current_job(&self) -> PyResult<PyJob> {
        let id = origen::with_current_job(|job| Ok(job.id))?;
        Ok(PyJob { id: id })
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
