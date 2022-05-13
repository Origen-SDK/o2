use crate::application;
use pyo3::prelude::*;
use pyapi_metal::framework::sessions::{Sessions, SessionStore, SessionGroup};
use origen::{
    with_app_session_group,
    with_app_session,
    om
};
use origen::utility::sessions::{clean_sessions, unload, setup_sessions, with_mut_app_session_group};

#[pymodule]
fn sessions(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<OrigenSessions>()?;
    Ok(())
}

#[pyclass(extends=Sessions, subclass)]
pub struct OrigenSessions {}

#[pymethods]
impl OrigenSessions {
    #[new]
    fn new() -> PyResult<(Self, Sessions)> {
        Ok((OrigenSessions {}, Sessions {}))
    }

    #[args(session = "None")]
    fn user_session(&self, session: Option<&PyAny>) -> PyResult<SessionStore> {
        // Can accept:
        //  None -> Top app's session
        //  String -> Session by name
        //  App -> (origen.application.Base) -> Session for this app/plugin
        let t;
        if let Some(s) = session {
            if let Ok(name) = s.extract::<String>() {
                t = Some(name);
            } else if application::is_base_app(s)? {
                let n = application::get_name(s)?;
                let mut sessions = om::sessions();
                om::with_current_user(|u| {
                    u.ensure_session(&mut sessions, Some(&n))
                })?;
                t = Some(n);
            } else {
                return crate::runtime_error!(format!(
                    "Could not derive session from input {}",
                    s.get_type().to_string()
                ));
            }
        } else {
            t = None;
        }

        Ok(om::with_current_user_session(t, |_, _, s| {
            Ok(SessionStore::from_metal(s)?)
        })?)
    }

    #[args(session = "None")]
    fn app_session(&self, session: Option<&PyAny>) -> PyResult<SessionStore> {
        let t;
        if let Some(s) = session {
            if let Ok(name) = s.extract::<String>() {
                t = Some(name);
            } else if application::is_base_app(s)? {
                let n = application::get_name(s)?;
                let sessions = om::sessions();
                with_mut_app_session_group(Some(sessions), |sg| {
                    sg.ensure(&n)
                })?;
                t = Some(n);
            } else {
                return crate::runtime_error!(format!(
                    "Could not derive session from input {}",
                    s.get_type().to_string()
                ));
            }
        } else {
            t = None;
        }

        Ok(with_app_session(t, |s| {
            Ok(SessionStore::from_metal(s)?)
        })?)
    }

    #[getter]
    pub fn user_sessions(&self) -> PyResult<SessionGroup> {
        Ok(om::with_current_user_session(None, |_, grp, _| {
            Ok(SessionGroup::from_metal(grp)?)
        })?)
    }

    #[getter]
    pub fn app_sessions(&self) -> PyResult<SessionGroup> {
        Ok(with_app_session_group(None, |grp, _| {
            Ok(SessionGroup::from_metal(grp)?)
        })?)
    }

    #[getter]
    fn user_session_root(&self, py: Python) -> PyResult<PyObject> {
        Ok(om::with_current_user_session(None, |_, grp, _| {
            Ok(pyapi_metal::pypath!(py, grp.path().display()))
        })?)
    }

    #[getter]
    fn app_session_root(&self, py: Python) -> PyResult<PyObject> {
        Ok(with_app_session_group(None, |grp, _| {
            Ok(pyapi_metal::pypath!(py, grp.path().display()))
        })?)
    }

    fn clean(&self) -> PyResult<()> {
        clean_sessions()?;
        Ok(())
    }

    fn setup(&self) -> PyResult<()> {
        setup_sessions()?;
        Ok(())
    }

    fn unload(&self) -> PyResult<()> {
        unload()?;
        Ok(())
    }
}
