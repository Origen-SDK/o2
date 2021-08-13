pub mod _frontend;

use super::app_utility;
use crate::runtime_error;
use origen::STATUS;
use pyo3::types::PyDict;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use origen::utility::release_scribe::ReleaseScribe as OrigenRS;
use origen::utility::version::Version as OVersion;

#[pymodule]
pub fn release_scribe(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ReleaseScribe>()?;
    m.add_wrapped(wrap_pyfunction!(app_release_scribe))?;
    Ok(())
}

#[pyfunction]
fn app_release_scribe() -> PyResult<Option<PyObject>> {
    let mut default;
    match &STATUS.app {
        Some(a) => {
            let c = a.config();
            app_utility(
                "release_scribe",
                match &c.release_scribe {
                    Some(config) => Some(config),
                    None => {
                        default = HashMap::new();
                        default.insert("system".to_string(), "origen.utility.release_scribe.ReleaseScribe".to_string());
                        Some(&default)
                    }
                },
                None::<fn(Option<&HashMap<String, String>>) -> PyResult<Option<PyObject>>>,
            )
        }
        None => {
            return runtime_error!(
                "Cannot retrieve the application's release_scribe config: no application found!"
            )
        }
    }
}

#[pyclass(subclass)]
pub struct ReleaseScribe {
    rs: OrigenRS
}

#[pymethods]
impl ReleaseScribe {
    #[new]
    #[args(config="**")]
    fn new(config: Option<&PyDict>) -> PyResult<Self> {
        let mut c: HashMap<String, String> = HashMap::new();
        if let Some(cfg) = config {
            for (k, v) in cfg {
                c.insert(k.extract::<String>()?, v.extract::<String>()?);
            }
        }
        Ok( Self {
            rs: OrigenRS::new(&c)?
        })
    }

    #[getter]
    fn release_note_file(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(crate::pypath!(py, format!("{}", self.rs.release_file.display())))
    }

    fn get_release_note(&self) -> PyResult<String> {
        Ok(self.rs.get_release_note()?)
    }

    fn get_release_note_from_file(&self) -> PyResult<String> {
        Ok(self.rs.get_release_note_from_file()?)
    }

    fn get_release_title(&self) -> PyResult<Option<String>> {
        Ok(self.rs.get_release_title()?)
    }

    #[getter]
    fn history_tracking_file(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(crate::pypath!(py, format!("{}", self.rs.history_toml.display())))
    }

    #[args(release="None", title="None", dry_run="false")]
    fn append_history(&mut self, body: String, title: Option<String>, release: Option<&PyAny>, dry_run: bool) -> PyResult<()> {
        let rel;
        match release {
            Some(r) => {
                if let Ok(s) = r.extract::<String>() {
                    // Since we're coming from Python, we'll assuming Pep-440 convention
                    rel = OVersion::new_pep440(&s)?;
                } else {
                    return type_error!("Could not extract 'release'!")
                }
            },
            None => {
                // Use the current version
                match &STATUS.app {
                    Some(a) => rel = a.version()?,
                    None => return runtime_error!("Could not get application version!")
                }
            }
        }
        self.rs.append_history(&rel, title, Some(body), dry_run)?;
        Ok(())
    }

    fn read_history(&self) -> PyResult<()> {
        println!("{:?}", self.rs.read_history()?);
        Ok(())
    }
}
