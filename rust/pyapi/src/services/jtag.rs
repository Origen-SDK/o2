use crate::extract_value;
use crate::model::Model;
use crate::unpack_transaction_options;
use origen::services::{jtag, Service};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

#[pyclass]
#[derive(Debug, Clone)]
pub struct JTAG {
    id: usize,
}

#[pymethods]
impl JTAG {
    #[new]
    fn new() -> Self {
        JTAG { id: 0 }
    }

    pub fn id(&self) -> PyResult<usize> {
        Ok(self.id)
    }

    // This is called automatically by Origen whenever a service is registered, the object
    // returned by this method is what get saved in the controller's services dict.
    fn set_model(&mut self, name: &str, model: &Model) -> PyResult<Self> {
        crate::dut::PyDUT::ensure_pins("dut")?;
        let mut dut = origen::dut();
        let mut services = origen::services();
        let id = services.next_id();
        let service = Service::JTAG(jtag::Service::new(
            &dut, id, None, // default IR size
            None, // tdi
            None, // tdo
            None, // toms
            None, // tclk
            None, // trstn
        )?);
        services.add_service(service);
        model.materialize_mut(&mut dut)?.add_service(name, id)?;
        self.id = id;
        Ok(self.clone())
    }

    fn reset(&self) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;
        jtag.reset(&dut)?;
        Ok(())
    }

    #[args(kwargs = "**")]
    fn write_dr(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let mut trans = value.to_write_transaction(&dut)?;
        unpack_transaction_options(&mut trans, kwargs)?;

        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;

        jtag.write_dr(&dut, &trans)?;
        Ok(self.clone())
    }

    #[args(kwargs = "**")]
    fn verify_dr(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let mut trans = value.to_verify_transaction(&dut)?;
        unpack_transaction_options(&mut trans, kwargs)?;

        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;

        jtag.verify_dr(&dut, &trans)?;
        Ok(self.clone())
    }

    #[args(kwargs = "**")]
    fn write_ir(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let mut trans = value.to_write_transaction(&dut)?;
        unpack_transaction_options(&mut trans, kwargs)?;

        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;

        jtag.write_ir(&dut, &trans)?;
        Ok(self.clone())
    }

    #[args(kwargs = "**")]
    fn verify_ir(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let mut trans = value.to_verify_transaction(&dut)?;
        unpack_transaction_options(&mut trans, kwargs)?;

        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;

        jtag.verify_ir(&dut, &trans)?;
        Ok(self.clone())
    }
}
