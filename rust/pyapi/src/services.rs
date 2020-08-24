pub mod swd;

use crate::extract_value;
use crate::model::Model;
use origen::services::{jtag, Service};
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pymodule]
/// Implements the module _origen.services in Python
pub fn services(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<JTAG>()?;
    m.add_class::<swd::SWD>()?;

    Ok(())
}

#[pyclass]
#[derive(Debug, Clone)]
struct JTAG {
    id: usize,
}

#[pymethods]
impl JTAG {
    #[new]
    fn new() -> Self {
        JTAG { id: 0 }
    }

    // This is called automatically by Origen whenever a service is registered, the object
    // returned by this method is what get saved in the controller's services dict.
    fn set_model(&mut self, name: &str, model: &Model) -> PyResult<JTAG> {
        let mut dut = origen::dut();
        let mut services = origen::services();
        let service = Service::JTAG(jtag::Service::new());
        let id = services.add_service(service);
        model.materialize_mut(&mut dut)?.add_service(name, id)?;
        self.id = id;
        Ok(self.clone())
    }

    #[args(size = "None")]
    fn write_ir(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<JTAG> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, size, &dut)?;
        let service = services.get_mut_service(self.id)?;
        if let Service::JTAG(jtag) = service {
            jtag.write_ir(value)?;
        }
        Ok(self.clone())
    }

    #[args(size = "None")]
    fn write_dr(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<JTAG> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, size, &dut)?;
        let service = services.get_mut_service(self.id)?;
        if let Service::JTAG(jtag) = service {
            jtag.write_dr(value)?;
        }
        Ok(self.clone())
    }

    #[args(size = "None")]
    fn verify_ir(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<JTAG> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, size, &dut)?;
        let service = services.get_mut_service(self.id)?;
        if let Service::JTAG(jtag) = service {
            jtag.verify_ir(value)?;
        }
        Ok(self.clone())
    }

    #[args(size = "None")]
    fn verify_dr(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<JTAG> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, size, &dut)?;
        let service = services.get_mut_service(self.id)?;
        if let Service::JTAG(jtag) = service {
            jtag.verify_dr(value)?;
        }
        Ok(self.clone())
    }
}
