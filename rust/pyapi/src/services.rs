use crate::model::Model;
use origen::services::{jtag, Service};
use pyo3::prelude::*;

#[pymodule]
/// Implements the module _origen.services in Python
pub fn services(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<JTAG>()?;

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
    fn new(_obj: &PyRawObject) -> Self {
        JTAG { id: 0 }
    }

    // This is called automatically by Origen whenever a service is registered, the object
    // returned by this method is what get saved in the controller's services dict.
    fn set_model(&mut self, name: &str, model: &Model) -> PyResult<JTAG> {
        let mut dut = origen::dut();
        let service = Service::JTAG(jtag::Service::new());
        let id = dut.add_service(service);
        model.materialize_mut(&mut dut)?.add_service(name, id)?;
        self.id = id;
        Ok(self.clone())
    }

    fn write_ir(&self) -> PyResult<JTAG> {
        let mut dut = origen::dut();
        let service = dut.get_mut_service(self.id)?;
        if let Service::JTAG(jtag) = service {
            jtag.write_ir();
        }
        Ok(self.clone())
    }
}
