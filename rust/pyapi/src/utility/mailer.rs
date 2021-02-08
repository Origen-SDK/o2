use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pymodule]
fn mailer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Mailer>()?;
    m.add_wrapped(wrap_pyfunction!(_mailer))?;
    Ok(())
}

#[pyfunction]
pub fn _mailer() -> PyResult<Mailer> {
    Ok(Mailer {})
}

/// Simple Python class that wraps the Origen's mailer
#[pyclass(subclass)]
pub struct Mailer {}

#[pymethods]
impl Mailer {
    #[getter]
    fn get_server(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        Ok(m.server.clone())
    }

    // #[setter]
    // fn set_server(&mut self, s: Option<String>) -> PyResult<()> {
    //     self.mailer.server = s;
    //     Ok(())
    // }

    // #[getter]
    // fn get_port(&self) -> PyResult<Option<usize>> {
    //     // Ok(self.mailer.port)
    // }

    // #[setter]
    // fn set_port(&mut self, p: Option<usize>) -> PyResult<()> {
    //     self.mailer.port = p;
    //     Ok(())
    // }

    // fn test(&self, to: Option<Vec<&PyAny>>) -> PyResult<()> {
    fn test(&self) -> PyResult<()> {
        let m = origen::mailer();
        Ok(m.test(None)?)
    }

    // #[getter]
    // fn port(&self) -> PyResult<String> {
    //     Ok(mailer.port.unwrap())
    // }

    // #[getter]
    // fn auth(&self) -> PyResult<String> {
    //     Ok(mailer.port.unwrap())
    // }

    // #[getter]
    // fn port(&self) -> PyResult<String> {
    //     Ok(mailer.port.unwrap())
    // }
    // #[getter]
    // fn port(&self) -> PyResult<String> {
    //     Ok(mailer.port.unwrap())
    // }
}
