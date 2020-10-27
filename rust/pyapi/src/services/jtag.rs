use crate::extract_value;
use crate::model::Model;
use pyo3::prelude::*;
use origen::services::{jtag, Service};
use crate::registers::bit_collection::BitCollection;
use pyo3::types::{PyAny, PyType, PyDict, PyTuple};
use crate::{unpack_transaction_options, resolve_transaction};

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
            & dut,
            id,
            None, // default IR size
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

    #[args(write_opts="**")]
    fn write_register(&self, bits: &PyAny, write_opts: Option<&PyDict>) -> PyResult<()> {
        // let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;

        // let value = extract_value(bits, Some(32), &dut)?;
        // let mut trans = value.to_write_transaction(&dut)?;
        // unpack_transaction_options(&mut trans, write_opts)?;
        let trans = resolve_transaction(&dut, bits, origen::TransactionAction::Write, write_opts)?;
        jtag.write_register(&dut, &services, &trans)?;
        Ok(())
    }

    #[args(verify_opts="**")]
    fn verify_register(&self, bits: &PyAny, verify_opts: Option<&PyDict>) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        let services = origen::services();
        let jtag = services.get_as_jtag(self.id)?;
        let value = extract_value(bits, Some(32), &dut)?;
        let mut trans = value.to_write_transaction(&dut)?;
        unpack_transaction_options(&mut trans, verify_opts)?;
        jtag.verify_register(&dut, &services, &trans)?;
        Ok(())
    }

    // #[args(size = "None")]
    // fn write_ir(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<Self> {
    //     let dut = origen::dut();
    //     let mut services = origen::services();
    //     let value = extract_value(bits_or_val, size, &dut)?;
    //     let service = services.get_mut_service(self.id)?;
    //     if let Service::JTAG(jtag) = service {
    //         jtag.write_ir(value)?;
    //     }
    //     Ok(self.clone())
    // }

    // #[args(size = "None")]
    // fn write_dr(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<Self> {
    //     let dut = origen::dut();
    //     let mut services = origen::services();
    //     let value = extract_value(bits_or_val, size, &dut)?;
    //     let service = services.get_mut_service(self.id)?;
    //     if let Service::JTAG(jtag) = service {
    //         jtag.write_dr(value)?;
    //     }
    //     Ok(self.clone())
    // }

    // #[args(size = "None")]
    // fn verify_ir(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<Self> {
    //     let dut = origen::dut();
    //     let mut services = origen::services();
    //     let value = extract_value(bits_or_val, size, &dut)?;
    //     let service = services.get_mut_service(self.id)?;
    //     if let Service::JTAG(jtag) = service {
    //         jtag.verify_ir(value)?;
    //     }
    //     Ok(self.clone())
    // }

    // #[args(size = "None")]
    // fn verify_dr(&self, bits_or_val: &PyAny, size: Option<u32>) -> PyResult<Self> {
    //     let dut = origen::dut();
    //     let mut services = origen::services();
    //     let value = extract_value(bits_or_val, size, &dut)?;
    //     let service = services.get_mut_service(self.id)?;
    //     if let Service::JTAG(jtag) = service {
    //         jtag.verify_dr(value)?;
    //     }
    //     Ok(self.clone())
    // }
}
