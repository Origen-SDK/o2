use crate::application::{get_pyapp, PyApplication};
use origen::utility::version::Version as OVersion;
use origen::Result as OResult;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

pub struct ReleaseScribe {}

impl origen::core::frontend::ReleaseScribe for ReleaseScribe {
    fn release_note_file(&self) -> OResult<PathBuf> {
        Ok(self.with_py_rs(|py, rs| {
            let r = rs.call_method0(py, "release_note_file")?;
            let r_py_str = r.call_method0(py, "__str__")?;
            let r_str = r_py_str.extract::<String>(py)?;
            Ok(PathBuf::from(r_str))
        })?)
    }

    fn get_release_note(&self) -> OResult<String> {
        Ok(self.with_py_rs(|py, rs| {
            let r = rs.call_method0(py, "get_release_note")?;
            Ok(r.extract::<String>(py)?)
        })?)
    }

    fn get_release_note_from_file(&self) -> OResult<String> {
        Ok(self.with_py_rs(|py, rs| {
            let r = rs.call_method0(py, "get_release_note_from_file")?;
            Ok(r.extract::<String>(py)?)
        })?)
    }

    fn get_release_title(&self) -> OResult<Option<String>> {
        Ok(self.with_py_rs(|py, rs| {
            let r = rs.call_method0(py, "get_release_title")?;
            Ok(r.extract::<Option<String>>(py)?)
        })?)
    }

    fn history_tracking_file(&self) -> OResult<PathBuf> {
        Ok(self.with_py_rs(|py, rs| {
            let r = rs.getattr(py, "history_tracking_file")?;
            let r_py_str = r.call_method0(py, "__str__")?;
            let r_str = r_py_str.extract::<String>(py)?;
            Ok(PathBuf::from(r_str))
        })?)
    }

    fn append_history(
        &self,
        version: &OVersion,
        title: Option<&str>,
        body: &str,
        dry_run: bool,
    ) -> OResult<()> {
        Ok(self.with_py_rs(|py, rs| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("body", body)?;
            kwargs.set_item("title", title)?;
            kwargs.set_item("release", version.to_string())?;
            kwargs.set_item("dry_run", dry_run)?;

            rs.call_method(py, "append_history", (), Some(kwargs))?;
            Ok(())
        })?)
    }

    // fn read_history(&self) -> OResult<Option<ReleaseHistory>> {
    //     Ok(self.with_py_rs( |py, rs| {
    //         let r = rs.call_method0(py, "release_note_file")?;
    //         let r_str = r.extract::<String>(py)?;
    //         Ok(PathBuf::from(r_str))
    //     })?)
    // }
}

impl ReleaseScribe {
    fn with_py_rs<T, F>(&self, mut func: F) -> PyResult<T>
    where
        F: FnMut(Python, PyObject) -> PyResult<T>,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let rs = PyApplication::_get_release_scribe(pyapp, py)?;
        func(py, rs)
    }
}
