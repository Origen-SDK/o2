use pyo3::prelude::*;
use pyo3::types::{PyType, PyDict, PyTuple};
use std::collections::HashMap;
use std::path::Path;
use origen::revision_control::git::Git as OrigenGit;
use origen::revision_control::RevisionControlAPI;
use crate::runtime_error;
use super::Status;

pub static PY_GIT_MOD_PATH: &str = "origen.utility.revision_control.git";

#[pymodule]
fn git(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Git>()?;
    Ok(())
}

#[pyclass(subclass)]
pub struct Git {
    config: HashMap<String, String>
}

#[pymethods]
impl Git {

    #[classmethod]
    fn __init__(_cls: &PyType, instance: &PyAny, config: Option<&PyDict>) -> PyResult<()> {
        let mut c: HashMap<String, String> = HashMap::new();
        if let Some(cfg) = config {
            for (k, v) in cfg {
                c.insert(k.extract::<String>()?, v.extract::<String>()?);
            }
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.config = c;
        Ok(())
    }

    #[new]
    fn new(config: Option<&PyDict>) -> PyResult<Self> {
        let mut c: HashMap<String, String> = HashMap::new();
        if let Some(cfg) = config {
            for (k, v) in cfg {
                c.insert(k.extract::<String>()?, v.extract::<String>()?);
            }
        }
        Ok(Self {
            config: c
        })
    }

    fn populate(&self, version: &str) -> PyResult<()> {
        Ok(self.rc()?.populate(version)?)
    }

    fn checkout(&self, force: bool, path: Option<&str>, version: &str) -> PyResult<bool> {
        let rusty_path;
        if let Some(p) = path {
            rusty_path = Some(Path::new(p));
        } else {
            rusty_path = None;
        }
        Ok(self.rc()?.checkout(force, rusty_path, version)?)
    }

    fn revert(&self, _path: &str) -> PyResult<()> {
        todo!();
        // Ok(self.rc()?.revert(Path::new(path))?)
    }

    fn status(&self, path: Option<&str>) -> PyResult<Status> {
        let rusty_path;
        if let Some(p) = path {
            rusty_path = Some(Path::new(p));
        } else {
            rusty_path = None;
        }
        Ok(Status {
            stat: self.rc()?.status(rusty_path)?
        })
    }

    #[args(kwargs="**")]
    fn tag(&self, tagname: &str, kwargs: Option<&PyDict>) -> PyResult<()> {
        let msg: Option<&str>;
        Ok(self.rc()?.tag(
            tagname,
            if let Some(kws) = kwargs {
                if let Some(f) = kws.get_item("force") {
                    f.extract::<bool>()?
                } else {
                    false
                }
            } else {
                false
            },
            if let Some(kws) = kwargs {
                if let Some(m) = kws.get_item("msg") {
                    msg = m.extract::<Option<&str>>()?;
                    msg
                } else {
                    None
                }
            } else {
                None
            }
        )?)
    }

    fn init(&self) -> PyResult<bool> {
        Ok(self.rc()?.init()?)
    }

    fn is_initialized(&self) -> PyResult<bool> {
        Ok(self.rc()?.is_initialized()?)
    }

    #[args(paths="*", kwargs="**")]
    fn checkin(&self, paths: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<String> {
        let msg;
        if let Some(kw) = kwargs {
            match kw.get_item("msg") {
                Some(m) => {
                    msg = m.extract::<String>()?;
                },
                None => return runtime_error!("A 'msg' is required for checkin operations")
            }
        } else {
            return runtime_error!("A 'msg' is required for checkin operations");
        }

        let mut rusty_paths = vec![];
        for path in paths {
            let p = path.extract::<&str>()?;
            rusty_paths.push(Path::new(p));
        }
        Ok(self.rc()?.checkin(Some(rusty_paths), &msg)?)
    }

    fn checkin_all(&self, msg: &str) -> PyResult<String> {
        Ok(self.rc()?.checkin(None, msg)?)
    }
}

impl Git {
    fn rc(&self) -> origen::Result<OrigenGit> {
        origen::revision_control::RevisionControl::git_from_config(&self.config)
    }
}