use crate::dut::PyDUT;
use crate::memory_map::MemoryMap;
use crate::register::{Register, Registers};
use origen::DUT;
use pyo3::class::basic::{CompareOp, PyObjectProtocol};
use pyo3::class::PyMappingProtocol;
use pyo3::exceptions::{AttributeError, KeyError, TypeError};
use pyo3::prelude::*;

/// Implements the user APIs dut[.sub_block].memory_map() and
/// dut[.sub_block].memory_maps
#[pymethods]
impl PyDUT {
    //fn create_address_block(
    //    &self,
    //    memory_map_id: usize,
    //    name: &str,
    //    base_address: Option<u64>,
    //    range: Option<u64>,
    //    width: Option<u64>,
    //    access: Option<&str>,
    //) -> PyResult<usize> {
    //    let acc: AccessType = match access {
    //        Some(x) => match x.parse() {
    //            Ok(y) => y,
    //            Err(msg) => return Err(exceptions::OSError::py_err(msg)),
    //        },
    //        None => AccessType::ReadWrite,
    //    };

    //    Ok(DUT.lock().unwrap().create_address_block(
    //        memory_map_id,
    //        name,
    //        base_address,
    //        range,
    //        width,
    //        Some(acc),
    //    )?)
    //}

    fn get_or_create_address_block(
        &self,
        memory_map_id: usize,
        name: &str,
    ) -> PyResult<AddressBlock> {
        let mut dut = DUT.lock().unwrap();
        let mm = dut.get_memory_map(memory_map_id)?;
        let id = match mm.get_address_block_id(name) {
            Ok(v) => v,
            Err(_) => dut.create_address_block(memory_map_id, name, None, None, None, None)?,
        };
        Ok(AddressBlock {
            id: id,
            name: name.to_string(),
        })
    }
}

/// Implements dut[.sub_block].memory_map["my_map"].address_block("my_block")
#[pymethods]
impl MemoryMap {
    fn address_block(&self, name: &str) -> PyResult<AddressBlock> {
        let id = origen::dut()
            .get_memory_map(self.id)?
            .get_address_block_id(name)?;
        Ok(AddressBlock {
            id: id,
            name: name.to_string(),
        })
    }
}

/// Implements the user API to work with a memory map's collection of address blocks, an instance
/// of this is returned by dut[.sub_block].memory_maps["my_map"].address_blocks
#[pyclass]
#[derive(Debug, Clone)]
pub struct AddressBlocks {
    /// The ID of the memory map which owns the contained address blocks
    pub memory_map_id: usize,
    /// Iterator index
    pub i: usize,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl AddressBlocks {
    fn len(&self) -> PyResult<usize> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        Ok(map.address_blocks.len())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        let keys: Vec<String> = map.address_blocks.keys().map(|x| x.to_string()).collect();
        Ok(keys)
    }

    fn values(&self) -> PyResult<Vec<AddressBlock>> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        let values: Vec<AddressBlock> = map
            .address_blocks
            .iter()
            .map(|(k, v)| AddressBlock {
                id: *v,
                name: k.to_string(),
            })
            .collect();
        Ok(values)
    }

    fn items(&self) -> PyResult<Vec<(String, AddressBlock)>> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        let items: Vec<(String, AddressBlock)> = map
            .address_blocks
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    AddressBlock {
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
impl AddressBlocks {}

#[pyproto]
impl PyMappingProtocol for AddressBlocks {
    fn __len__(&self) -> PyResult<usize> {
        self.len()
    }

    /// Implements address_blocks["my_block"]
    fn __getitem__(&self, query: &str) -> PyResult<AddressBlock> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        if map.address_blocks.contains_key(query) {
            Ok(AddressBlock {
                id: map.get_address_block_id(query)?,
                name: query.to_string(),
            })
        } else {
            Err(KeyError::py_err(format!(
                "'{}' does not have an address block called '{}'",
                map.name, query
            )))
        }
    }
}

#[pyproto]
impl PyObjectProtocol for AddressBlocks {
    /// Implements address_blocks.my_block
    fn __getattr__(&self, query: &str) -> PyResult<AddressBlock> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        if map.address_blocks.contains_key(query) {
            Ok(AddressBlock {
                id: map.get_address_block_id(query)?,
                name: query.to_string(),
            })
        } else {
            Err(AttributeError::py_err(format!(
                "'AddressBlocks' object has no attribute '{}'",
                query
            )))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        let model = map.model(&dut)?;
        let (mut output, offset) = model.console_header(&dut);
        output += &(" ".repeat(offset));
        output += &format!("└── memory_maps['{}']\n", map.name);
        let mut leader = " ".repeat(offset + 5);
        output += &format!("{}└── address_blocks\n", leader);
        leader += "     ";
        let num = map.address_blocks.keys().len();
        if num > 0 {
            let mut keys: Vec<&String> = map.address_blocks.keys().collect();
            keys.sort();
            for (i, key) in keys.iter().enumerate() {
                if i != num - 1 {
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
impl pyo3::class::iter::PyIterProtocol for AddressBlocks {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<AddressBlocks> {
        let mut m = slf.clone();
        m.i = 0;
        Ok(m)
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        let dut = origen::dut();
        let map = dut.get_memory_map(slf.memory_map_id)?;
        let keys: Vec<&String> = map.address_blocks.keys().collect();
        // TODO: Sort this (and same for memory_maps equivalent method)

        if slf.i >= keys.len() {
            return Ok(None);
        }

        let id = keys[slf.i];
        slf.i += 1;
        Ok(Some(id.to_string()))
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for AddressBlocks {
    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let dut = origen::dut();
        let map = dut.get_memory_map(self.memory_map_id)?;
        Ok(map.address_blocks.contains_key(item))
    }
}

/// Implements the user API to work with a single address block
#[pyclass]
#[derive(Debug)]
pub struct AddressBlock {
    #[pyo3(get)]
    pub id: usize,
    #[pyo3(get)]
    pub name: String,
}

/// User API methods, available to both Rust and Python
#[pymethods]
impl AddressBlock {
    fn reg(&self, name: &str) -> PyResult<Option<Register>> {
        let id = origen::dut()
            .get_address_block(self.id)?
            .get_register_id(name);
        match id {
            Ok(id) => Ok(Some(Register {
                id: id,
                name: name.to_string(),
            })),
            Err(_) => Ok(None),
        }
    }
}

/// Internal, Rust-only methods
impl AddressBlock {}

#[pyproto]
impl PyObjectProtocol for AddressBlock {
    fn __getattr__(&self, query: &str) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let dut = origen::dut();

        if query == "regs" || query == "registers" {
            let pyref = PyRef::new(
                py,
                Registers {
                    address_block_id: Some(self.id),
                    register_file_id: None,
                    ids: None,
                    i: 0,
                },
            )?;
            Ok(pyref.to_object(py))

        // See if the requested attribute is a reference to one of this block's registers
        } else {
            let blk = dut.get_address_block(self.id)?;

            match blk.get_register_id(query) {
                Ok(id) => {
                    let pyref = PyRef::new(
                        py,
                        Register {
                            id: id,
                            name: query.to_string(),
                        },
                    )?;
                    Ok(pyref.to_object(py))
                }
                Err(_) => Err(AttributeError::py_err(format!(
                    "'AddressBlock' object has no attribute '{}'",
                    query
                ))),
            }
        }
    }

    fn __richcmp__(&self, other: &AddressBlock, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.id == other.id && self.name == other.name),
            CompareOp::Ne => Ok(self.id != other.id || self.name != other.name),
            CompareOp::Lt => Err(TypeError::py_err(
                "'<' not supported between instances of 'AddressBlock'",
            )),
            CompareOp::Le => Err(TypeError::py_err(
                "'<=' not supported between instances of 'AddressBlock'",
            )),
            CompareOp::Gt => Err(TypeError::py_err(
                "'>' not supported between instances of 'AddressBlock'",
            )),
            CompareOp::Ge => Err(TypeError::py_err(
                "'>=' not supported between instances of 'AddressBlock'",
            )),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let dut = origen::dut();
        let blk = dut.get_address_block(self.id)?;
        Ok(blk.console_display(&dut)?)
    }
}
