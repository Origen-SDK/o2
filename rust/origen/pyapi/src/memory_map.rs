// This module may be removed soon, replaced by the top-level DUT APIs
use crate::dut::PyDUT;
use origen::DUT;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::PyMappingProtocol;
use pyo3::prelude::*;
use pyo3::exceptions::{KeyError, AttributeError};

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    fn memory_maps(&self, path: &str) -> PyResult<MemoryMaps> {
        // Verify the model exists, though we don't need it for now
        DUT.lock().unwrap().get_model(path)?;
        Ok(MemoryMaps {
            model_path: path.to_string(),
        })
    }

    fn memory_map(&self, path: &str, id: &str) -> PyResult<MemoryMap> {
        // Verify the model exists, though we don't need it for now
        DUT.lock().unwrap().get_model(path)?;
        Ok(MemoryMap {
            model_path: path.to_string(),
            id: id.to_string(),
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

/// User API methods, available to both Rust and Python
#[pymethods]
impl MemoryMaps {
    fn len(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        Ok(model.memory_maps.len())
    }
}

/// Internal, Rust-only methods
impl MemoryMaps {
}

#[pyproto]
impl PyMappingProtocol for MemoryMaps {
    fn __len__(&self) -> PyResult<usize> {
        self.len()
    }

    /// Implements memory_map["my_map"]
    fn __getitem__(&self, query: &str) -> PyResult<MemoryMap> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        if model.memory_maps.contains_key(query) {
            Ok(MemoryMap {
                model_path: self.model_path.clone(),
                id: query.to_string(),
            })
        } else {
            Err(KeyError::py_err(format!("'{}' does not have a memory map called '{}'", model.display_path, query)))
        }
    }
}

#[pyproto]
impl PyObjectProtocol for MemoryMaps {
    /// Implements memory_map.my_map
    fn __getattr__(&self, query: &str) -> PyResult<MemoryMap> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        if model.memory_maps.contains_key(query) {
            Ok(MemoryMap {
                model_path: self.model_path.clone(),
                id: query.to_string(),
            })
        } else {
            Err(AttributeError::py_err(format!("'MemoryMaps' object has no attribute '{}'", query)))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(&self.model_path)?;
        //let mut output: String = "".to_string();
        let (mut output, offset) = model.console_header();
        output += &(" ".repeat(offset));
        output += "└── memory_maps\n";
        let leader = " ".repeat(offset + 5);
        let num_maps = model.memory_maps.keys().len();
        if num_maps > 0 {
            for (i, key) in model.memory_maps.keys().enumerate() {
                if i != num_maps - 1 {
                    output += &format!("{}├── {}\n", leader, key);
                } else {
                    output += &format!("{}└── {}\n", leader, key);
                }
            }
        } else {
            output += &format!("{}└── NONE\n", leader);
        }
        Ok(output)
    }
}

//#[pyproto]
//impl PyIterProtocol for MemoryMaps {
//    fn __iter__(slf: PyRefMut<Self>) -> PyResult<MemoryMap> {
//        let py = unsafe { Python::assume_gil_acquired() };
//        Ok(slf.to_object(py))
//    }
//    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
//        let py = unsafe { Python::assume_gil_acquired() };
//        match slf.keys.as_ref(py).as_bytes().get(slf.idx) {
//            Some(&b) => {
//                let res = slf
//                    .reader
//                    .as_ref(py)
//                    .inner
//                    .get(&b)
//                    .map(|s| PyString::new(py, s).into());
//                slf.idx += 1;
//                Ok(res)
//            }
//            None => Ok(None),
//        }
//    }
//}

/// Implements the user API to work with a single memory map, an instance
/// of this is returned by dut[.sub_block].memory_maps["my_map"]
#[pyclass]
#[derive(Debug)]
pub struct MemoryMap {
    /// The path to the model which owns the contained memory maps
    model_path: String,
    id: String,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl MemoryMap {
}

/// Internal, Rust-only methods
impl MemoryMap {}

#[pyproto]
impl PyObjectProtocol for MemoryMap {
    fn __repr__(&self) -> PyResult<String> {
        Ok("Here should be a nice graphic of the memory map".to_string())
    }
}
