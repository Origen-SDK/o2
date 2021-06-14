use indexmap::IndexMap;
use origen::Result;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyTuple};
use std::collections::HashMap;
use crate::utility::metadata::{extract_as_metadata, metadata_to_pyobj};
use crate::with_pycallbacks;

pub struct Frontend {}

impl Frontend {
    pub fn new() -> Self {
        Self {}
    }
}

impl origen::core::frontend::Frontend for Frontend {
    fn app(&self) -> origen::Result<Option<Box<dyn origen::core::frontend::App>>> {
        let app_frontend = crate::application::_frontend::App::new()?;
        Ok(Some(Box::new(app_frontend)))
    }

    fn emit_callback(
        &self,
        callback: &str,
        args: Option<&Vec<origen::Metadata>>,
        kwargs: Option<&IndexMap<String, origen::Metadata>>,
        // source: Option<String>,
        _opts: Option<&HashMap<String, origen::Metadata>>,
    ) -> origen::Result<Vec<origen::Metadata>> {
        Ok(with_pycallbacks(|py, cbs| {
            let pyargs = PyTuple::new(
                py,
                vec![
                    callback.to_object(py),
                    {
                        let v: Vec<PyObject> = vec![];
                        let py_args = PyList::new(py, v);
                        if let Some(_args) = args {
                            for arg in _args {
                                py_args.append(metadata_to_pyobj(Some(arg.clone()), None)?)?;
                            }
                        }
                        py_args.to_object(py)
                    },
                    {
                        let py_kwargs = PyDict::new(py);
                        if let Some(_kwargs) = kwargs {
                            for (kw, arg) in _kwargs {
                                py_kwargs
                                    .set_item(kw, metadata_to_pyobj(Some(arg.clone()), None)?)?;
                            }
                        }
                        py_kwargs.to_object(py)
                    },
                ],
            );
            let pykwargs = PyDict::new(py);
            let r = cbs.call_method("emit", pyargs, Some(pykwargs))?;

            let pyretn = r.extract::<Vec<&PyAny>>()?;
            let mut retn = vec![];
            for i in pyretn {
                retn.push(extract_as_metadata(i)?);
            }
            Ok(retn)
        })?)
    }

    fn register_callback(&self, callback: &str, _description: &str) -> origen::Result<()> {
        with_pycallbacks(|py, cbs| {
            cbs.call_method("register_callback", PyTuple::new(py, &[callback]), None)?;
            Ok(())
        })?;
        Ok(())
    }

    fn list_local_dependencies(&self) -> origen::Result<Vec<String>> {
        todo!()
    }

    fn on_dut_change(&self) -> Result<()> {
        with_pycallbacks(|_py, cbs| {
            cbs.call_method0("unload_on_dut_change")?;
            Ok(())
        })?;
        Ok(())
    }
}
