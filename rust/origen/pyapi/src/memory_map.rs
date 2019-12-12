// This module may be removed soon, replaced by the top-level DUT APIs
use crate::dut::PyDUT;
use origen::DUT;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::PyMappingProtocol;
use pyo3::exceptions;
use pyo3::prelude::*;

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    fn memory_maps(&self, path: &str) -> PyResult<MemoryMaps> {
        let dut = DUT.lock().unwrap();
        // Verify this model exists, though we don't need it for now
        match dut.get_model(path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        Ok(MemoryMaps {
            model_path: path.to_string(),
        })
    }
}

/// Implements the user API to work with a model's collection of memory maps, an instance
/// of this is returned by dut[.sub_block].memory_maps
#[pyclass]
#[derive(Debug)]
pub struct MemoryMaps {
    /// The path to the model which owns the contained memory maps
    model_path: String,
}

#[pymethods]
impl MemoryMaps {
    fn len(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        // Verify this model exists, though we don't need it for now
        let model = match dut.get_model(&self.model_path) {
            Ok(m) => m,
            Err(e) => return Err(exceptions::OSError::py_err(e.msg)),
        };
        Ok(model.memory_maps.len())
    }
}

#[pyproto]
impl PyMappingProtocol for MemoryMaps {
    fn __len__(&self) -> PyResult<usize> {
        self.len()
    }
}

#[pyproto]
impl PyObjectProtocol for MemoryMaps {
    fn __repr__(&self) -> PyResult<String> {
        Ok("Here should be a nice graphic of the memory maps".to_string())
    }
}
