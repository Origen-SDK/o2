use pyo3::prelude::*;

#[pymodule]
pub fn application(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyApplication>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyApplication {}

#[pymethods]
impl PyApplication {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({ PyApplication {} });
    }

    #[getter]
    fn version(&self) -> PyResult<String> {
        Ok(format!("{}", origen::app().unwrap().version()?))
    }
}
