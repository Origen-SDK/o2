use crate::with_pycallbacks;
use origen::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyapi_metal::prelude::typed_value;
use typed_value::{TypedValueVec, TypedValueMap};

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
        args: Option<&TypedValueVec>,
        kwargs: Option<&TypedValueMap>,
        // source: Option<String>,
        _opts: Option<&TypedValueMap>,
    ) -> origen::Result<TypedValueVec> {
        Ok(with_pycallbacks(|py, cbs| {
            let pyargs = PyTuple::new(
                py,
                vec![
                    callback.to_object(py),
                    {
                        if let Some(l) = args {
                            typed_value::into_pylist(py, &mut l.typed_values().iter())?.to_object(py)
                        } else {
                            PyList::empty(py).to_object(py)
                        }
                    },
                    {
                        typed_value::option_into_pydict(py, kwargs)?.to_object(py)
                    },
                ],
            );
            let pykwargs = PyDict::new(py);
            let r = cbs.call_method("emit", pyargs, Some(pykwargs))?;

            let pyretn = r.extract::<&PyList>()?;
            Ok(typed_value::from_pylist(pyretn)?)
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
