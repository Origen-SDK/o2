use super::address_block::{AddressBlock, AddressBlocks};
use super::register_collection::RegisterCollection;
use crate::dut::PyDUT;
use origen::DUT;
use pyo3::class::basic::{CompareOp, PyObjectProtocol};
use pyo3::class::PyMappingProtocol;
use pyo3::exceptions::{PyAttributeError, PyKeyError, PyTypeError};
use pyo3::prelude::*;

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    fn memory_maps(&self, model_id: usize) -> PyResult<MemoryMaps> {
        Ok(MemoryMaps {
            model_id: model_id,
            i: 0,
        })
    }

    fn memory_map(&self, model_id: usize, name: &str) -> PyResult<MemoryMap> {
        let id = DUT
            .lock()
            .unwrap()
            .get_model(model_id)?
            .get_memory_map_id(name)?;
        Ok(MemoryMap {
            id: id,
            name: name.to_string(),
        })
    }

    fn create_memory_map(
        &self,
        model_id: usize,
        name: &str,
        address_unit_bits: Option<u32>,
    ) -> PyResult<usize> {
        Ok(DUT
            .lock()
            .unwrap()
            .create_memory_map(model_id, name, address_unit_bits)?)
    }

    fn get_or_create_memory_map(&self, model_id: usize, name: &str) -> PyResult<MemoryMap> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_model(model_id)?;
        let id = match model.get_memory_map_id(name) {
            Ok(v) => v,
            Err(_) => dut.create_memory_map(model_id, name, None)?,
        };
        Ok(MemoryMap {
            id: id,
            name: name.to_string(),
        })
    }
}

/// Implements the user API to work with a model's collection of memory maps, an instance
/// of this is returned by dut[.sub_block].memory_maps
#[pyclass]
#[derive(Debug, Clone)]
pub struct MemoryMaps {
    /// The ID of the model which owns the contained memory maps
    model_id: usize,
    /// Iterator index
    i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl MemoryMaps {
    fn len(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        Ok(model.memory_maps.len())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        let keys: Vec<String> = model.memory_maps.keys().map(|x| x.clone()).collect();
        Ok(keys)
    }

    fn values(&self) -> PyResult<Vec<MemoryMap>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        let values: Vec<MemoryMap> = model
            .memory_maps
            .iter()
            .map(|(k, v)| MemoryMap {
                id: *v,
                name: k.to_string(),
            })
            .collect();
        Ok(values)
    }

    fn items(&self) -> PyResult<Vec<(String, MemoryMap)>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        let items: Vec<(String, MemoryMap)> = model
            .memory_maps
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    MemoryMap {
                        id: *v,
                        name: k.to_string(),
                    },
                )
            })
            .collect();
        Ok(items)
    }
}

/// Internal, Rust-only methods
impl MemoryMaps {}

#[pyproto]
impl PyMappingProtocol for MemoryMaps {
    fn __len__(&self) -> PyResult<usize> {
        self.len()
    }

    /// Implements memory_map["my_map"]
    fn __getitem__(&self, query: &str) -> PyResult<MemoryMap> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        if model.memory_maps.contains_key(query) {
            Ok(MemoryMap {
                id: model.get_memory_map_id(query)?,
                name: query.to_string(),
            })
        } else {
            Err(PyKeyError::new_err(format!(
                "'{}' does not have a memory map called '{}'",
                model.display_path(&dut),
                query
            )))
        }
    }
}

#[pyproto]
impl PyObjectProtocol for MemoryMaps {
    /// Implements memory_map.my_map
    fn __getattr__(&self, query: &str) -> PyResult<MemoryMap> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        if model.memory_maps.contains_key(query) {
            Ok(MemoryMap {
                id: model.get_memory_map_id(query)?,
                name: query.to_string(),
            })
        } else {
            Err(PyAttributeError::new_err(format!(
                "'MemoryMaps' object has no attribute '{}'",
                query
            )))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        let (mut output, offset) = model.console_header(&dut);
        output += &(" ".repeat(offset));
        output += "└── memory_maps\n";
        let leader = " ".repeat(offset + 5);
        let num_maps = model.memory_maps.keys().len();
        if num_maps > 0 {
            let mut keys: Vec<&String> = model.memory_maps.keys().collect();
            keys.sort();
            for (i, key) in keys.iter().enumerate() {
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

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for MemoryMaps {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<MemoryMaps> {
        let mut m = slf.clone();
        m.i = 0;
        Ok(m)
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(slf.model_id).unwrap();
        let keys: Vec<&String> = model.memory_maps.keys().collect();

        if slf.i >= keys.len() {
            return Ok(None);
        }

        let id = keys[slf.i];
        slf.i += 1;
        Ok(Some(id.to_string()))
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for MemoryMaps {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let dut = DUT.lock().unwrap();
        let model = dut.get_model(self.model_id)?;
        Ok(model.memory_maps.contains_key(item))
    }
}

/// Implements the user API to work with a single memory map, an instance
/// of this is returned by dut[.sub_block].memory_maps["my_map"]
#[pyclass]
#[derive(Debug)]
pub struct MemoryMap {
    #[pyo3(get)]
    pub id: usize,
    #[pyo3(get)]
    pub name: String,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl MemoryMap {}

/// Internal, Rust-only methods
impl MemoryMap {}

#[pyproto]
impl PyObjectProtocol for MemoryMap {
    fn __getattr__(&self, query: &str) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let dut = origen::dut();

        // Calling .regs on an individual memory map returns the regs in its default
        // address block (the one named 'default').
        // If a default address block has not been defined then an empty Registers collection
        // is returned.
        if query == "regs" {
            let ab_id = dut.get_memory_map(self.id)?.get_address_block_id("default");
            let pyobj = Py::new(
                py,
                RegisterCollection {
                    address_block_id: match ab_id {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    },
                    register_file_id: None,
                    ids: None,
                    i: 0,
                },
            )?;

            Ok(pyobj.to_object(py))
        } else if query == "address_blocks" {
            Ok(Py::new(
                py,
                AddressBlocks {
                    memory_map_id: self.id,
                    i: 0,
                },
            )?
            .to_object(py))

        // See if the requested attribute is a reference to one of this map's address blocks
        } else {
            let map = dut.get_memory_map(self.id)?;

            match map.get_address_block_id(query) {
                Ok(id) => {
                    let pyobj = Py::new(
                        py,
                        AddressBlock {
                            id: id,
                            name: query.to_string(),
                        },
                    )?;
                    Ok(pyobj.to_object(py))
                }
                Err(_) => Err(PyAttributeError::new_err(format!(
                    "'MemoryMap' object has no attribute '{}'",
                    query
                ))),
            }
        }
    }

    fn __richcmp__(&self, other: PyRef<'p, MemoryMap>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.id == other.id && self.name == other.name),
            CompareOp::Ne => Ok(self.id != other.id || self.name != other.name),
            CompareOp::Lt => Err(PyTypeError::new_err(
                "'<' not supported between instances of 'MemoryMap'",
            )),
            CompareOp::Le => Err(PyTypeError::new_err(
                "'<=' not supported between instances of 'MemoryMap'",
            )),
            CompareOp::Gt => Err(PyTypeError::new_err(
                "'>' not supported between instances of 'MemoryMap'",
            )),
            CompareOp::Ge => Err(PyTypeError::new_err(
                "'>=' not supported between instances of 'MemoryMap'",
            )),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let dut = origen::dut();
        let memory_map = dut.get_memory_map(self.id)?;
        Ok(memory_map.console_display(&dut)?)
    }
}
