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
    fn get_server(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.get_server()?)
    }

    // #[setter]
    // fn set_server(&mut self, s: Option<String>) -> PyResult<()> {
    //     self.mailer.server = s;
    //     Ok(())
    // }

    #[getter]
    fn get_port(&self) -> PyResult<Option<usize>> {
        let m = origen::mailer();
        Ok(m.port)
    }

    // #[setter]
    // fn set_port(&mut self, p: Option<usize>) -> PyResult<()> {
    //     self.mailer.port = p;
    //     Ok(())
    // }

    #[getter]
    fn get_domain(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        Ok(m.domain.clone())
    }

    #[getter]
    fn get_auth_method(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.auth_method.to_string())
    }

    #[getter]
    fn get_timeout_seconds(&self) -> PyResult<u64> {
        let m = origen::mailer();
        Ok(m.timeout_seconds)
    }

    #[getter]
    fn get_timeout(&self) -> PyResult<u64> {
        let m = origen::mailer();
        Ok(m.timeout_seconds)
    }

    #[getter]
    fn get_service_user(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        if let Some(su) = m.service_user()? {
            Ok(Some(su.0.to_string()))
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn get_username(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.username()?)
    }
    #[getter]
    fn get_password(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.password()?)
    }

    #[getter]
    fn get_sender(&self) -> PyResult<String> {
        let m = origen::mailer();
        Ok(m.sender()?)
    }

    #[getter]
    fn get_dataset(&self) -> PyResult<Option<String>> {
        let m = origen::mailer();
        Ok(m.dataset()?)
    }

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
