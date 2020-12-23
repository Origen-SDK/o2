use pyo3::prelude::*;
use origen::utility::mailer::Mailer as OrigenMailer;

// #[pyfunction]
// fn email_server(py: Python) -> PyResult<Option<String>> {
//     Ok(origen::app().unwrap().email_server())
// }

// #[pyfunction]
// fn email_port(py: Python)
// #[pyfunction]
// fn email_auth(py: Python)
// #[pyfunction]
// fn email_domain(py: Python)
// #[pyfunction]
// fn email_username(py: Python)
// #[pyfunction]
// fn email_password(py: Python)

// #[pyfunction]
// send()
// #[pyfunction]
// test()

/// Simple Python class that wraps the Origen's mailer
#[pyclass(subclass)]
pub struct Mailer {
    mailer: OrigenMailer
}

#[pymethods]
impl Mailer {

    #[new]
    fn new(id: String) -> PyResult<Self> {
        Ok(Self {
            mailer: OrigenMailer::new()?
        })
    }

    #[getter]
    fn get_server(&self) -> PyResult<Option<String>> {
        Ok(self.mailer.server.clone())
    }

    #[setter]
    fn set_server(&mut self, s: Option<String>) -> PyResult<()> {
        self.mailer.server = s;
        Ok(())
    }

    #[getter]
    fn get_port(&self) -> PyResult<Option<usize>> {
        Ok(self.mailer.port)
    }

    #[setter]
    fn set_port(&mut self, p: Option<usize>) -> PyResult<()> {
        self.mailer.port = p;
        Ok(())
    }

    fn test(&self, to: Option<Vec<&PyAny>>) -> PyResult<()> {
        Ok(self.mailer.test()?)
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