use origen_metal::utils::mailer::Mailer as OMailer;
use origen_metal::utils::mailer::PASSWORD_MOTIVE as OM_PASSWORD_MOTIVE;
use origen_metal::utils::mailer::MailerTOMLConfig;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use crate::prelude::{PyOutcome, typed_value};
use crate::prelude::users::*;

// TEST_NEEDED
pub const OM_MAILER_CLASS_QP: &str = "origen_metal.utils.mailer.Mailer";

/// Simple Python class that wraps the Origen's mailer
#[pyclass(subclass)]
pub struct Mailer {
    mailer: OMailer,
}

#[pymethods]
impl Mailer {
    #[new]
    #[args(timeout="60")]
    pub fn new(server: String, port: Option<u16>, domain: Option<String>, auth_method: Option<&str>, timeout: Option<u64>, user: Option<String>) -> PyResult<Self> {
        Ok(Self {
            mailer: OMailer::new(server, port, domain, auth_method, timeout, user)?
        })
    }

    #[getter]
    fn config<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        typed_value::into_pydict(py, self.mailer.config()?)
    }

    #[getter]
    fn get_server(&self) -> PyResult<String> {
        Ok(self.mailer.server.clone())
    }

    #[getter]
    fn get_port(&self) -> PyResult<Option<u16>> {
        Ok(self.mailer.port)
    }

    #[getter]
    fn get_domain(&self) -> PyResult<Option<String>> {
        Ok(self.mailer.domain.clone())
    }

    #[getter]
    fn get_auth_method(&self) -> PyResult<Option<String>> {
        if self.mailer.auth_method.is_none() {
            Ok(None)
        } else {
            Ok(Some(self.mailer.auth_method.to_string()))
        }
    }

    #[getter]
    fn get_timeout_seconds(&self) -> PyResult<u64> {
        Ok(self.mailer.timeout_seconds)
    }

    #[getter]
    fn get_timeout(&self) -> PyResult<u64> {
        Ok(self.mailer.timeout_seconds)
    }

    #[getter]
    fn get_user(&self) -> PyResult<Option<PyUser>> {
        if let Some(s) = self.mailer.user()? {
            let users = users()?;
            Ok(Some(users.require_user(s)?))
        } else {
            users()?.current_user()
        }
    }

    #[getter]
    fn __user__(&self) -> PyResult<Option<&String>> {
        Ok(self.mailer.user()?)
    }

    #[getter]
    fn get_username(&self) -> PyResult<String> {
        Ok(self.mailer.username()?)
    }

    #[getter]
    fn get_password(&self) -> PyResult<String> {
        Ok(self.mailer.password()?)
    }

    #[getter]
    fn get_sender(&self) -> PyResult<String> {
        Ok(self.mailer.sender()?)
    }

    #[getter]
    fn get_dataset(&self) -> PyResult<Option<String>> {
        Ok(self.mailer.dataset()?)
    }

    fn test(&self, to: Option<Vec<&str>>) -> PyResult<PyOutcome> {
        Ok(PyOutcome::from_origen(self.mailer.test(to)?))
    }

    fn send(
        &self,
        to: Vec<&str>,
        body: Option<&str>,
        subject: Option<&str>,
    ) -> PyResult<PyOutcome> {
        let e = origen_metal::require_current_user_email()?;
        let m = self.mailer.compose(&e, to, subject, body, true)?;
        Ok(PyOutcome::from_origen(self.mailer.send(m)?))
    }

    // TODO?
    // #[getter]
    // fn signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn origen_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn app_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    // #[getter]
    // fn release_signature(&self) -> PyResult<Option<String>> {
    //     //
    // }

    #[classattr]
    const PASSWORD_MOTIVE: &'static str = OM_PASSWORD_MOTIVE;
}

impl Mailer {
    pub fn toml_config_into_args<'py>(py: Python<'py>, config: &MailerTOMLConfig) -> PyResult<&'py PyTuple> {
        Ok(PyTuple::new(py, [
            config.server.to_object(py),
            config.port.to_object(py),
            config.domain.to_object(py),
            config.auth_method.to_object(py),
            config.timeout.to_object(py),
            config.user.to_object(py)
        ]))
    }
}