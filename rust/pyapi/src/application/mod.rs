pub mod _frontend;

use crate::runtime_error;
use crate::utility::results::{BuildResult};
use origen_metal::utils::version::Version as OVersion;
use pyapi_metal::prelude::*;
use pyapi_metal::utils::revision_control::status::Status;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::path::{Path, PathBuf};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "application")?;
    subm.add_class::<PyApplication>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyApplication {}

#[pymethods]
impl PyApplication {
    #[new]
    #[pyo3(signature=(**_kwargs))]
    fn new(_kwargs: Option<&PyDict>) -> Self {
        PyApplication {}
    }

    #[getter]
    fn version(&self) -> PyResult<String> {
        let v = origen::app().unwrap().version()?.to_string();
        Ok(format!(
            "{}",
            OVersion::new_pep440(&v)?.to_string()
        ))
    }

    fn check_production_status(&self) -> PyResult<bool> {
        let r = origen::app().unwrap().check_production_status(false)?;
        Ok(r.passed())
    }

    #[pyo3(signature=(**kwargs))]
    fn __publish__(&self, kwargs: Option<&PyDict>) -> PyResult<PyOutcome> {
        let mut dry_run = false;
        let mut rn: Option<&str> = None;
        let mut rt: Option<Option<&str>> = None;
        let mut ver: Option<OVersion> = None;
        if let Some(kw) = kwargs {
            if let Some(d) = kw.get_item("dry-run") {
                dry_run = d.extract::<bool>()?;
            }
            if let Some(r) = kw.get_item("release-note") {
                rn = Some(r.extract::<&str>()?);
            }
            if let Some(r) = kw.get_item("release-title") {
                rt = Some(Some(r.extract::<&str>()?));
            }
            if let Some(_r) = kw.get_item("no-release-title") {
                if rt.is_some() {
                    return runtime_error!(
                        "A release title cannot be given along with option 'no-release-title'"
                    );
                } else {
                    rt = Some(None);
                }
            }
            if let Some(v) = kw.get_item("version") {
                ver = Some(OVersion::new_pep440(&v.extract::<String>()?)?);
            }
        }
        Ok(PyOutcome::from_origen(
            origen::app().unwrap().publish(ver, rt, rn, dry_run)?,
        ))
    }

    fn __rc_init__(&self) -> PyResult<PyOutcome> {
        Ok(PyOutcome::from_origen(origen::app().unwrap().rc_init()?))
    }

    fn __rc_status__(&self) -> PyResult<Status> {
        Ok(Status::from_origen(origen::app().unwrap().rc_status()?))
    }

    #[pyo3(signature=(pathspecs, msg, dry_run))]
    fn __rc_checkin__(
        &self,
        pathspecs: Option<Vec<String>>,
        msg: &str,
        dry_run: bool,
    ) -> PyResult<PyOutcome> {
        let mut paths = vec![];
        if let Some(ps) = pathspecs.as_ref() {
            paths = ps.iter().map(|p| Path::new(p)).collect();
        }

        Ok(PyOutcome::from_origen(origen::app().unwrap().rc_checkin(
            {
                if pathspecs.is_some() {
                    Some(paths)
                } else {
                    None
                }
            },
            msg,
            dry_run,
        )?))
    }

    #[pyo3(signature=(*_args))]
    fn __build_package__(&self, _args: &PyTuple) -> PyResult<BuildResult> {
        Ok(BuildResult {
            build_result: Some(origen::app().unwrap().build_package()?),
        })
    }

    #[pyo3(signature=(*_args))]
    fn __run_publish_checks__(&self, _args: &PyTuple) -> PyResult<bool> {
        let r = origen::app().unwrap().run_publish_checks(false)?;
        Ok(r.passed())
    }
}

impl PyApplication {
    pub fn _get_rc<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        log_trace!("Retrieving application's RC...");
        let r = slf.as_ref(py).getattr("rc")?;
        if r.is_none() {
            return crate::runtime_error!("No RC is available on the application");
        }

        log_trace!("Retrieved application RC");
        Ok(r.to_object(py))
    }

    pub fn _get_ut<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).getattr("unit_tester")?;
        if r.is_none() {
            return crate::runtime_error!("No unit tester is available on the application");
        }

        Ok(r.to_object(py))
    }

    pub fn _get_publisher<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).getattr("publisher")?;
        if r.is_none() {
            return crate::runtime_error!("No publisher is available on the application");
        }

        Ok(r.to_object(py))
    }

    pub fn _get_linter<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).getattr("linter")?;
        if r.is_none() {
            return crate::runtime_error!("No Linter is available on the application");
        }

        Ok(r.to_object(py))
    }

    pub fn _get_website<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).call_method0("website")?;
        if r.is_none() {
            return runtime_error!("No website is available on the application");
        }

        Ok(r.to_object(py))
    }

    pub fn _get_release_scribe<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).getattr("release_scribe")?;
        if r.is_none() {
            return runtime_error!("No release_scribe is available on the application");
        }

        Ok(r.to_object(py))
    }

    pub fn _get_mailer<'py>(slf: Py<Self>, py: Python<'py>) -> PyResult<PyObject> {
        let r = slf.as_ref(py).getattr("mailer")?;
        if r.is_none() {
            return runtime_error!("No mailer is available on the application");
        }

        Ok(r.to_object(py))
    }
}

pub fn get_pyapp<'py>(py: Python<'py>) -> PyResult<Py<PyApplication>> {
    log_trace!("Retrieving PyApplication object from Python heap...");
    let locals = PyDict::new(py);
    locals.set_item("origen", py.import("origen")?.to_object(py))?;
    let result = py.eval("origen.app", Some(locals), None)?;

    if result.is_none() {
        return runtime_error!("No Origen application is present");
    }

    match result.extract::<Py<PyApplication>>() {
        Ok(app) => {
            log_trace!("Retrieved PyApplication object");
            Ok(app)
        }
        Err(_e) => runtime_error!(
            "'origen.app' points to an object which cannot be extracted as an Origen application"
        ),
    }
}

// pub struct Linter {}

// impl ofrontend::Linter for Linter {
//     fn run(&self) -> OResult<LinterStatus> {
//         // ...
//     }
// }

// pub struct Website {}

// impl ofrontend::Website for Website {
//     fn build(&self) -> OResult<WebBuildStatus> {
//         // ...
//     }
// }

/* The Base application is implemented mostly in Python, but has some relevant
   properties usable in Rust.

   Below are some functions to grab data from an assumed origen.application.Base instance
*/

/// Query if the current object is an instance of origen.application.Base
/// Note: this could have several methods overridden. Just check that the aforementioned
/// class is one of the object's ancestors
pub fn is_base_app(query: &PyAny) -> PyResult<bool> {
    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?.to_object(py))?;
        locals.set_item("builtins", py.import("builtins")?.to_object(py))?;
        locals.set_item("query", query.to_object(py))?;
        let result = py.eval(
            "builtins.isinstance(query, origen.application.Base)",
            Some(locals),
            None,
        )?;
        Ok(result.extract::<bool>()?)
    })
}

/// Return the name of the given app. Equivalent to `app.name` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_name(app: &PyAny) -> PyResult<String> {
    if is_base_app(app)? {
        Ok(app.getattr("name")?.extract::<String>()?)
    } else {
        crate::runtime_error!("Cannot get name of non-origen.application.Base object")
    }
}

#[allow(dead_code)]
/// Return the root path of the given app. Equivalent to `app.root` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_root(app: &PyAny) -> PyResult<PathBuf> {
    if is_base_app(app)? {
        let p = app.getattr("root")?.extract::<String>()?;
        Ok(PathBuf::from(p))
    } else {
        crate::runtime_error!("Cannot get root of non-origen.application.Base object")
    }
}

#[allow(dead_code)]
/// Return the app path of the given app. Equivalent to `app.app_dir` in Python
/// Returns an error if the given object isn't a `origen.application.Base`
pub fn get_app_dir(app: &PyAny) -> PyResult<PathBuf> {
    if is_base_app(app)? {
        let p = app.getattr("app_dir")?.extract::<String>()?;
        Ok(PathBuf::from(p))
    } else {
        crate::runtime_error!("Cannot get root of non-origen.application.Base object")
    }
}
