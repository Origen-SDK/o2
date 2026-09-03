use crate::model::Model;
use crate::resolve_transaction;
use origen::services::{simple, Service};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Simple {
    id: Option<usize>,

    // Temporarily store the arguments, then clean up after initialized
    args: Option<(String, String, String, usize)>,
}

#[pymethods]
impl Simple {
    #[new]
    fn new(clk: &str, data: &str, read_nwrite: &str, width: usize) -> Self {
        Self {
            id: None,
            args: Some((
                clk.to_string(),
                data.to_string(),
                read_nwrite.to_string(),
                width,
            )),
        }
    }

    pub fn set_model(&mut self, name: &str, model: &Model) -> PyResult<Self> {
        // crate::dut::PyDUT::ensure_pins("dut")?;
        let mut dut = origen::dut();
        let mut services = origen::services();
        let id = services.next_id();
        let service;
        match self.args.as_ref() {
            Some(args) => {
                let clk_pin = dut._get_pin_group(0, &args.0)?;
                let data_bus = dut._get_pin_group(0, &args.1)?;
                let read_nwrite_pin = dut._get_pin_group(0, &args.2)?;
                service = Service::Simple(simple::Service::new(
                    &dut,
                    id,
                    clk_pin,
                    data_bus,
                    read_nwrite_pin,
                    args.3
                )?);
            },
            None => return crate::runtime_error!(
                "Protocol Simple has not been properly initialized - missing initialization arguments"
            )
        }
        services.add_service(service);
        model.materialize_mut(&mut dut)?.add_service(name, id)?;
        self.id = Some(id);
        self.args = None;
        Ok(self.clone())
    }

    fn reset(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = origen::dut();
        let services = origen::services();
        let service = services.get_as_simple(slf.id()?)?;
        service.reset(&dut)?;
        Ok(slf.into())
    }

    #[pyo3(signature=(bits_or_val, **write_opts))]
    fn write_register(
        slf: PyRef<Self>,
        bits_or_val: &PyAny,
        write_opts: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        let dut = origen::dut();
        let services = origen::services();
        let trans = resolve_transaction(
            &dut,
            bits_or_val,
            Some(origen::TransactionAction::Write),
            write_opts,
        )?;

        let simple = services.get_as_simple(slf.id()?)?;
        simple.write(&dut, trans)?;
        Ok(slf.into())
    }

    #[pyo3(signature=(bits_or_val, **verify_opts))]
    fn verify_register(
        slf: PyRef<Self>,
        bits_or_val: &PyAny,
        verify_opts: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        let dut = origen::dut();
        let services = origen::services();
        let trans = resolve_transaction(
            &dut,
            bits_or_val,
            Some(origen::TransactionAction::Verify),
            verify_opts,
        )?;

        let simple = services.get_as_simple(slf.id()?)?;
        simple.verify(&dut, trans)?;
        Ok(slf.into())
    }
}

impl Simple {
    fn id(&self) -> PyResult<usize> {
        match self.id {
            Some(id) => Ok(id),
            None => crate::runtime_error!("Protocol 'Simple' has not been properly initialized"),
        }
    }
}
