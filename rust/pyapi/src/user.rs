use origen::core::user::User as OrigenUser;
use pyo3::{wrap_pyfunction};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use super::utility::session_store::{SessionStore, user_session};

#[pymodule]
fn users(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(current_user))?;
    m.add_class::<User>()?;
    Ok(())
}

#[pyfunction]
fn current_user() -> PyResult<User> {
    User::new(origen::current_user!().id().unwrap())
}

// #[pyfunction]
// fn switch_user() -> PyResult<()> {
//     // ...
// }

// #[pyfunction]
// fn users() -> PyResult<Vec<User>> {
//     // ...
// }

macro_rules! user {
    ($user: ident) => {{
        origen::current_user!()
    }};
}

#[pyclass(subclass)]
pub struct User {
    user_id: String
}

#[pymethods]
impl User {

    // #[classmethod]
    // fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
    //     Ok(())
    // }

    #[new]
    fn new(id: String) -> PyResult<Self> {
        Ok(Self {
            user_id: id
        })
    }

    // #[getter]
    // fn get_current(&self) -> PyResult<bool> {
    //     Ok(self.user()?.current())
    // }

    #[getter]
    fn get_id(&self) -> PyResult<String> {
        Ok(user!(self).id().unwrap())
    }

    #[getter]
    fn get_username(&self) -> PyResult<Option<String>> {
        Ok(user!(self).name())
    }

    #[getter]
    fn get_name(&self) -> PyResult<Option<String>> {
        self.get_username()
    }

    #[getter]
    fn get_email(&self) -> PyResult<Option<String>> {
        Ok(user!(self).email())
    }

    #[getter]
    fn get_password(&self) -> PyResult<String> {
        Ok(user!(self).password(None, None)?)
    }

    // fn authenticate(&self) -> PyResult<()> {
    //     Ok(self.user()?.authenticate()?)
    // }

    #[getter]
    fn authenticated(&self) -> PyResult<bool> {
        Ok(user!(self).authenticated())
    }

    #[getter]
    fn authentication_failed(&self) -> PyResult<bool> {
        Ok(user!(self).authentication_failed())
    }

    // fn switch_to() -> PyResult<()> {
    //     // ...
    // }

    #[getter]
    fn home_dir(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(crate::pypath!(py, user!(self).home_dir_string()?))
    }

    #[getter]
    fn session(&self) -> PyResult<SessionStore> {
        user_session(None)
    }
}

// impl User {
//     fn user(&self) -> &OrigenUser {
//         //Ok(origen::STATUS.get_user_by_id(self.user_id)?)
//         origen::current_user!()
//     }
// }

// impl PartialEq for User {
//     // ...
// }

// impl Debug for User {
//     // ...
// }

// impl Display for User {
//     // ...
// }