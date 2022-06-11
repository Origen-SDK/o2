use super::bit_collection::BitCollection;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;

/// Implements the user API to work with a collection of registers. The collection could be associated
/// with another container object (an address block or register file), or this could be its own collection
/// of otherwise un-related registers.
#[pyclass]
#[derive(Debug, Clone)]
pub struct RegisterCollection {
    /// The ID of the address block which contains these registers. It is optional so that
    /// an empty RegisterCollection can be created, or an ad-hoc collection registers
    pub address_block_id: Option<usize>,
    /// The ID of the register file which contains these registers. It is optional as registers
    /// can be instantiated in an address block directly and are not necessarily within a
    /// register file
    pub register_file_id: Option<usize>,
    /// The IDs of the contained registers. If not present then the IDs will be derived from either
    /// the associated register file or address block. If both are defined then the register file IDs
    /// will be used.
    pub ids: Option<Vec<usize>>,
    /// Iterator index
    pub i: usize,
}

impl RegisterCollection {
    pub fn new() -> RegisterCollection {
        RegisterCollection {
            address_block_id: None,
            register_file_id: None,
            ids: None,
            i: 0,
        }
    }
}

//#[pyproto]
//impl pyo3::class::iter::PyIterProtocol for TimesetContainer {
//    fn __iter__(slf: PyRefMut<Self>) -> PyResult<DictLikeIter> {
//        DictLikeAPI::__iter__(&*slf)
//    }
//}

/// User API methods, available to both Rust and Python
#[pymethods]
impl RegisterCollection {
    fn len(&self) -> PyResult<usize> {
        if let Some(x) = &self.ids {
            return Ok(x.len());
        }
        let dut = origen::dut();
        if let Some(x) = self.register_file_id {
            return Ok(dut.get_register_file(x)?.registers.len());
        }
        if let Some(x) = self.address_block_id {
            return Ok(dut.get_address_block(x)?.registers.len());
        }
        Ok(0)
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = origen::dut();
        if let Some(ids) = &self.ids {
            let mut keys: Vec<String> = Vec::new();
            for id in ids {
                keys.push(dut.get_register(*id)?.name.clone());
            }
            return Ok(keys);
        }
        if let Some(x) = self.register_file_id {
            let rf = dut.get_register_file(x)?;
            let keys: Vec<String> = rf.registers.keys().map(|x| x.clone()).collect();
            return Ok(keys);
        }
        if let Some(x) = self.address_block_id {
            let ab = dut.get_address_block(x)?;
            let keys: Vec<String> = ab.registers.keys().map(|x| x.clone()).collect();
            return Ok(keys);
        }
        Ok(Vec::new())
    }

    fn values(&self) -> PyResult<Vec<BitCollection>> {
        let dut = origen::dut();
        if let Some(ids) = &self.ids {
            let values: Vec<BitCollection> = ids
                .iter()
                .map(|x| BitCollection::from_reg_id(*x, &dut))
                .collect();
            return Ok(values);
        }
        if let Some(x) = self.register_file_id {
            let rf = dut.get_register_file(x)?;
            let values: Vec<BitCollection> = rf
                .registers
                .values()
                .map(|x| BitCollection::from_reg_id(*x, &dut))
                .collect();
            return Ok(values);
        }
        if let Some(x) = self.address_block_id {
            let ab = dut.get_address_block(x)?;
            let values: Vec<BitCollection> = ab
                .registers
                .values()
                .map(|x| BitCollection::from_reg_id(*x, &dut))
                .collect();
            return Ok(values);
        }
        Ok(Vec::new())
    }

    fn items(&self) -> PyResult<Vec<(String, BitCollection)>> {
        let dut = origen::dut();
        if let Some(ids) = &self.ids {
            let mut items: Vec<(String, BitCollection)> = Vec::new();
            for id in ids {
                items.push((
                    dut.get_register(*id)?.name.clone(),
                    BitCollection::from_reg_id(*id, &dut),
                ));
            }
            return Ok(items);
        }
        if let Some(x) = self.register_file_id {
            let rf = dut.get_register_file(x)?;
            let items: Vec<(String, BitCollection)> = rf
                .registers
                .iter()
                .map(|(k, v)| (k.to_string(), BitCollection::from_reg_id(*v, &dut)))
                .collect();
            return Ok(items);
        }
        if let Some(x) = self.address_block_id {
            let ab = dut.get_address_block(x)?;
            let items: Vec<(String, BitCollection)> = ab
                .registers
                .iter()
                .map(|(k, v)| (k.to_string(), BitCollection::from_reg_id(*v, &dut)))
                .collect();
            return Ok(items);
        }
        Ok(Vec::new())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<RegisterCollection: {:?}>", self.keys().unwrap()))
    }

    fn __contains__(&self, name: &str) -> PyResult<bool> {
        match self.__getitem__(name) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn __getitem__(&self, name: &str) -> PyResult<BitCollection> {
        let dut = origen::dut();
        if let Some(ids) = &self.ids {
            for id in ids {
                let reg = dut.get_register(*id)?;
                if reg.name == name {
                    return Ok(BitCollection::from_reg_id(*id, &dut));
                }
            }
            return Err(PyKeyError::new_err(format!(
                "The register collection does not contain a register named '{}'",
                name
            )));
        }
        if let Some(x) = self.register_file_id {
            let rf = dut.get_register_file(x)?;
            let id = rf.get_register_id(name)?;
            return Ok(BitCollection::from_reg_id(id, &dut));
        }
        if let Some(x) = self.address_block_id {
            let ab = dut.get_address_block(x)?;
            let id = ab.get_register_id(name)?;
            return Ok(BitCollection::from_reg_id(id, &dut));
        }
        Err(PyKeyError::new_err(format!(
            "Register collection does not contain a register named '{}'",
            name
        )))
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.len()?)
    }
}
