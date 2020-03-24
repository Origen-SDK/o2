use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use origen::{STATUS};
use std::path::{PathBuf};

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
  pub fn run(&self) -> PyResult<()> {
    let cmd;
    {
      let p = origen::producer();
      let j = &p.jobs[self.id];
      cmd = j.command.clone();
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = [("origen", py.import("origen")?)].into_py_dict(py);
    println!("{}", &cmd);
    py.eval(
      &format!(
        "origen.load_file(r\"{}\", locals={{**origen.standard_context(), **{{'produce_pattern': lambda **kwargs : __import__(\"origen\").producer.produce_pattern(__import__(\"origen\").producer.get_job_by_id({}), **kwargs)}}}})",
        &cmd,
        self.id),
      None, 
      Some(locals)
    )?;
    Ok(())
  }

  #[getter]
  pub fn id(&self) -> PyResult<usize> {
    Ok(self.id)
  }

  #[getter]
  pub fn command(&self) -> PyResult<String> {
    let p = origen::producer();
    let j = &p.jobs[self.id];
    Ok(j.command.clone())
  }
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyProducer {
}

#[pymethods]
impl PyProducer {
  #[new]
  fn new(obj: &PyRawObject) {
    obj.init({ PyProducer {}
    });
  }

  fn create_pattern_job(&self, command: &str) -> PyResult<Py<PyJob>> {
    let mut p = origen::producer();
    //let mut path = PathBuf::new();
    //path.push(command);
    let j = p.create_pattern_job(command)?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(Py::new(py, PyJob {
      id: j.id,
    }).unwrap())
  }

  fn get_job_by_id(&self, id: usize) -> PyResult<Py<PyJob>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(Py::new(py, PyJob {
      id: id,
    }).unwrap())
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
    obj.init({ PyPattern {}
    });
  }
}
