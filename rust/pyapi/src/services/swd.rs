use origen::services::{swd, Service};
use origen::services::swd::Acknowledgements;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::extract_value;
use crate::model::Model;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct SWD {
    id: usize,
}

#[pymethods]
impl SWD {
    #[new]
    fn new(_obj: &PyRawObject) -> Self {
        Self { id: 0 }
    }

    fn set_model(&mut self, name: &str, model: &Model) -> PyResult<Self> {
        let mut dut = origen::dut();
        let mut services = origen::services();
        let service = Service::SWD(swd::Service::new());
        let id = services.add_service(service);
        model.materialize_mut(&mut dut)?.add_service(name, id)?;
        self.id = id;
        Ok(self.clone())
    }

    #[args(kwargs="**")]
    fn write_ap(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let service = services.get_mut_service(self.id)?;
        let (mut ap_addr, mut ack) = (0 as u32, Acknowledgements::Ok);
        if let Some(args) = kwargs {
            if let Some(_ap_addr) = args.get_item("ap_addr") {
                ap_addr = _ap_addr.extract::<u32>()?;
            }
            if let Some(_ack) = args.get_item("acknowledge") {
                ack = Acknowledgements::from_str(&_ack.extract::<String>()?)?;
            }
        }
        if let Service::SWD(swd) = service {
            swd.write_ap(value, ap_addr, ack)?;
        }
        Ok(self.clone())
    }

    #[args(kwargs="**")]
    fn verify_ap(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let service = services.get_mut_service(self.id)?;
        let (mut ap_addr, mut ack, mut parity) = (0 as u32, Acknowledgements::Ok, None);
        if let Some(args) = kwargs {
            if let Some(_ap_addr) = args.get_item("ap_addr") {
                ap_addr = _ap_addr.extract::<u32>()?;
            }
            if let Some(_ack) = args.get_item("acknowledge") {
                ack = Acknowledgements::from_str(&_ack.extract::<String>()?)?;
            }
            if let Some(_parity) = args.get_item("parity") {
                parity = Some(_parity.extract::<u32>()? != 0);
            }
        }
        if let Service::SWD(swd) = service {
            swd.verify_ap(value, ap_addr, ack, parity)?;
        }
        Ok(self.clone())
    }

    #[args(kwargs="**")]
    fn write_dp(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let service = services.get_mut_service(self.id)?;
        let (mut dp_addr, mut ack) = (0 as u32, Acknowledgements::Ok);
        if let Some(args) = kwargs {
            if let Some(_dp_addr) = args.get_item("dp_addr") {
                dp_addr = _dp_addr.extract::<u32>()?;
            }
            if let Some(_ack) = args.get_item("acknowledge") {
                ack = Acknowledgements::from_str(&_ack.extract::<String>()?)?;
            }
        }
        if let Service::SWD(swd) = service {
            swd.write_dp(value, dp_addr, ack)?;
        }
        Ok(self.clone())
    }

    #[args(kwargs="**")]
    fn verify_dp(&self, bits_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        let mut services = origen::services();
        let value = extract_value(bits_or_val, Some(32), &dut)?;
        let service = services.get_mut_service(self.id)?;
        let (mut dp_addr, mut ack, mut parity) = (0 as u32, Acknowledgements::Ok, None);
        if let Some(args) = kwargs {
            if let Some(_dp_addr) = args.get_item("dp_addr") {
                dp_addr = _dp_addr.extract::<u32>()?;
            }
            if let Some(_ack) = args.get_item("acknowledge") {
                ack = Acknowledgements::from_str(&_ack.extract::<String>()?)?;
            }
            if let Some(_parity) = args.get_item("parity") {
                parity = Some(_parity.extract::<u32>()? != 0);
            }
        }
        if let Service::SWD(swd) = service {
            swd.verify_dp(value, dp_addr, ack, parity)?;
        }
        Ok(self.clone())
    }

    fn line_reset(&self) -> PyResult<Self> {
        let mut services = origen::services();
        let service = services.get_mut_service(self.id)?;
        if let Service::SWD(swd) = service {
            swd.line_reset()?;
        }
        Ok(self.clone())
    }

    // Enum-like Acknowledgments

    #[allow(non_snake_case)]
    #[classmethod]
    fn OK(_cls: &PyType) -> PyResult<&str> {
        Ok("Ok")
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn WAIT(_cls: &PyType) -> PyResult<&str> {
        Ok("Wait")
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn FAULT(_cls: &PyType) -> PyResult<&str> {
        Ok("Fault")
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn NONE(_cls: &PyType) -> PyResult<&str> {
        Ok("None")
    }
}
