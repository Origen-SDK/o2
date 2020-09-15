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
        obj.init(PyApplication {});
    }

    #[getter]
    fn version(&self) -> PyResult<String> {
        let v = origen::app().unwrap().version()?.to_string();
        Ok(format!("{}", origen::utility::version::to_pep440(&v)?))
    }
}
