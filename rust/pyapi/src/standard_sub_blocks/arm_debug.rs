use origen::standard_sub_blocks::ArmDebug as OrigenArmDebug;
use origen::standard_sub_blocks::arm_debug::mem_ap::MemAP as OrigenMemAP;
use pyo3::prelude::*;
use crate::registers::bit_collection::BitCollection;
use pyo3::types::{PyAny, PyType, PyDict, PyTuple};
use num_bigint::BigUint;

#[pymodule]
/// Implements the module _origen.standard_sub_blocks in Python
pub fn arm_debug(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArmDebug>()?;
    m.add_class::<MemAP>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Clone)]
/// Backend controller connecting the :link-to:`origen.standard_sub_blocks.ArmDebug` view
/// with the :link-to:`origen::standard_sub_blocks::ArmDebug` model.
/// The controller here is responsible for instantiating and initializing the
/// ArmDebug model.
pub struct ArmDebug {}

impl ArmDebug {
    fn _add_mem_ap(instance: &PyObject, name: &str, ap: Option<u32>, csw_reset: Option<u32>) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let args = PyTuple::new(py, &[name.to_object(py)]);
        let kwargs = PyDict::new(py);
        kwargs.set_item("mod_path", "origen.standard_sub_blocks.arm_debug.mem_ap".to_object(py))?;
        let sb_options = PyDict::new(py);
        if let Some(_ap) = ap {
            sb_options.set_item("ap", _ap)?;
        }
        if let Some(_csw_reset) = csw_reset {
            sb_options.set_item("csw_reset", _csw_reset)?;
        }
        kwargs.set_item("sb_options", sb_options.to_object(py))?;

        //instance.call_method(py, "add_sub_block", args, Some(kwargs))?;
        Ok(())
    }
}

#[pymethods]
impl ArmDebug {

    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({
            Self {}
        });
    }

    #[classmethod]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        {
            let mut dut = origen::dut();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            OrigenArmDebug::model_init(&mut dut, model_id)?;
        }

        if let Some(opts) = block_options {
            if let Some(mem_aps) = opts.get_item("mem_aps") {
                let aps = mem_aps.downcast_ref::<PyDict>()?;
                for (ap_name, ap_opts) in aps.iter() {
                    let ap_opts_dict = ap_opts.downcast_ref::<PyDict>()?;
                    Self::_add_mem_ap(
                        &obj,
                        &ap_name.extract::<String>()?,
                        {
                            if let Some(ap_addr) = ap_opts_dict.get_item("ap") {
                                Some(ap_addr.extract::<u32>()?)
                            } else {
                                None
                            }
                        },
                        {
                            if let Some(csw_reset) = ap_opts_dict.get_item("csw_reset") {
                                Some(csw_reset.extract::<u32>()?)
                            } else {
                                None
                            }
                        }
                    )?;
                }
            }
        }
        Ok(())
    }

    fn add_mem_ap(slf: PyRef<Self>, name: &str, ap: Option<u32>, csw_reset: Option<u32>) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = slf.to_object(py);
        Self::_add_mem_ap(&obj, name, ap, csw_reset)
    }
}

#[pyclass(subclass)]
struct MemAP {
    //ub mem_ap: Option<OrigenMemAP>,
}

#[pymethods]
impl MemAP {

    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new(obj: &PyRawObject) -> Self {
            Self { 
                //mem_ap: None
            }
    }

    #[classmethod]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        let addr;
        //let csw_reset;
        if let Some(ap_opts_dict) = block_options {
            if let Some(ap_addr) = ap_opts_dict.get_item("ap") {
                addr = BigUint::from(ap_addr.extract::<u64>()?);
            } else {
                addr = BigUint::from(0 as u32);
            }
        } else {
            addr = BigUint::from(0 as u32);
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        let args = PyTuple::new(py, &["default".to_object(py), "default".to_object(py)]);
        //let mem_ap;
        {
            let mut dut = origen::dut();
            //let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            //mem_ap = OrigenMemAP::model_init(&mut dut, model_id, addr)?;
        }
        let slf = instance.downcast_mut::<Self>()?;
        //slf.mem_ap = Some(mem_ap);
        //obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    /// Initiates an ArmDebug MemAP write based on the given register (passed in as
    /// a BitCollection).
    /// Assumes that all posturing has been completed - that is, the bits' data, overlay
    /// status, etc. is current.
    fn write_register(&self, bits: &PyAny, latency: Option<u32>) -> PyResult<()> {
        let bc = bits.downcast_ref::<BitCollection>()?;
        let dut = origen::dut();
        //self.mem_ap.as_ref().unwrap().write_register(&dut, &bc.materialize(&dut)?)?;
        Ok(())
    }
}
