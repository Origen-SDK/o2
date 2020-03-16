use origen::core::model::Model as RichModel;
use origen::Dut;
use origen::Result;
use pyo3::prelude::*;
use std::sync::MutexGuard;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Model {
    pub id: usize,
}

impl Model {
    pub fn new(id: usize) -> Model {
        Model { id: id }
    }

    /// Turn into a full Model
    pub fn materialize<'a>(&self, dut: &'a MutexGuard<Dut>) -> Result<&'a RichModel> {
        dut.get_model(self.id)
    }

    /// Turn into a full Model
    pub fn materialize_mut<'a>(&self, dut: &'a mut MutexGuard<Dut>) -> Result<&'a mut RichModel> {
        dut.get_mut_model(self.id)
    }
}

#[pymethods]
impl Model {
    #[getter]
    /// This returns the offset attribute that was supplied when instantiating the block.
    /// To get the blocks fully resolved address, use the address() method instead.
    fn offset(&self) -> PyResult<u128> {
        Ok(self.materialize(&origen::dut())?.offset)
    }

    #[getter]
    fn address_unit_bits(&self) -> PyResult<u32> {
        Ok(self.materialize(&origen::dut())?.address_unit_bits)
    }

    /// Returns the fully resolved address of the block which is comprised of the sum of
    /// it's own offset and that of it's parent(s).
    fn address(&self, _address_unit_bits: Option<u32>) -> PyResult<u128> {
        let dut = origen::dut();
        Ok(self.materialize(&dut)?.address(&dut)?)
    }

    #[getter]
    fn id(&self) -> PyResult<usize> {
        Ok(self.id)
    }
}
