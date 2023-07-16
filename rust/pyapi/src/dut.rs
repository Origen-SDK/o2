use crate::model::Model as ModelProxy;
use crate::registers::RegisterCollection;
use origen::Error;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};
use pyo3::wrap_pymodule;

//TODO is this needed/used?
#[allow(dead_code)]
pub fn get_pydut(py: Python) -> PyResult<&PyAny> {
    let locals = PyDict::new(py);
    locals.set_item("origen", py.import("origen")?.to_object(py))?;
    Ok(py.eval("origen.dut", Some(locals), None)?)
}

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "dut")?;
    subm.add_class::<PyDUT>()?;
    crate::pins::define(py, subm)?;
    crate::registers::define(py, subm)?;
    crate::timesets::define(py, subm)?;
    m.add_submodule(subm)?;
    Ok(())
}

/// The PyDUT object, through which DUT-related interactions between the Python frontend and the Rust backend take place.
#[pyclass]
#[derive(Debug)]
pub struct PyDUT {
    metadata: Vec<PyObject>,
}

#[pymethods]
impl PyDUT {
    #[new]
    /// Instantiating a new instance of PyDUT means re-loading the target
    fn new(name: &str) -> PyResult<Self> {
        origen::dut().change(name)?;
        origen::services().change();
        Ok(PyDUT { metadata: vec![] })
    }

    /// Creates a new model at the given path
    fn create_model(
        &self,
        parent_id: Option<usize>,
        name: &str,
        offset: Option<u128>,
    ) -> PyResult<usize> {
        Ok(origen::dut().create_model(parent_id, name, offset)?)
    }

    fn model_console_display(&self, model_id: usize) -> PyResult<String> {
        let dut = origen::dut();
        let model = dut.get_model(model_id)?;
        Ok(model.console_display(&dut)?)
    }

    /// push_metadata(self, item)
    /// Pushes metadata object onto the current DUT
    pub fn push_metadata(&mut self, item: &PyAny) -> usize {
        let gil = Python::acquire_gil();
        let py = gil.python();

        self.metadata.push(item.to_object(py));
        self.metadata.len() - 1
    }

    pub fn override_metadata_at(&mut self, idx: usize, item: &PyAny) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if self.metadata.len() > idx {
            self.metadata[idx] = item.to_object(py);
            Ok(())
        } else {
            Err(PyErr::from(Error::new(&format!(
                "Overriding metadata at {} exceeds the size of the current metadata vector!",
                idx
            ))))
        }
    }

    pub fn get_metadata(&self, idx: usize) -> PyResult<&PyObject> {
        Ok(&self.metadata[idx])
    }

    pub fn model(&self, id: usize) -> PyResult<ModelProxy> {
        Ok(ModelProxy::new(id))
    }

    pub fn empty_regs(&self) -> PyResult<RegisterCollection> {
        Ok(RegisterCollection::new())
    }
}

impl PyDUT {
    pub fn ensure_pins(model_path: &str) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?.to_object(py))?;
        py.eval(&format!("origen.{}.pins", model_path), Some(locals), None)?;
        Ok(())
    }
}
