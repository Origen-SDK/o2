use origen::standard_sub_blocks::ArmDebug as OrigenArmDebug;
use origen::standard_sub_blocks::arm_debug::DP as OrigenDP;
use origen::standard_sub_blocks::arm_debug::mem_ap::MemAP as OrigenMemAP;
use pyo3::prelude::*;
use crate::registers::bit_collection::BitCollection;
use pyo3::types::{PyAny, PyType, PyDict, PyTuple};
use num_bigint::BigUint;
use pyo3::ToPyObject;

#[pymodule]
/// Implements the module _origen.standard_sub_blocks in Python
pub fn arm_debug(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArmDebug>()?;
    m.add_class::<DP>()?;
    m.add_class::<MemAP>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Clone)]
/// Backend controller connecting the :link-to:`origen.standard_sub_blocks.ArmDebug` view
/// with the :link-to:`origen::standard_sub_blocks::ArmDebug` model.
/// The controller here is responsible for instantiating and initializing the
/// ArmDebug model.
pub struct ArmDebug {
    arm_debug: Option<OrigenArmDebug>,
}

#[pymethods]
impl ArmDebug {

    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
            Self {
                arm_debug: None
            }
    }

    #[classmethod]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Create the Arm Debug instance
        let arm_debug;
        {
            let mut dut = origen::dut();
            let model_id = instance.getattr("model_id")?.extract::<usize>()?;
            arm_debug = OrigenArmDebug::model_init(&mut dut, model_id)?;
        }

        // Add the DP subblock
        let args = PyTuple::new(py, &["dp".to_object(py)]);
        let kwargs = PyDict::new(py);
        kwargs.set_item("mod_path", "origen.standard_sub_blocks.arm_debug.dp".to_object(py))?;
        instance.downcast::<PyCell<Self>>()?.call_method("add_sub_block", args, Some(kwargs))?;

        if let Some(opts) = block_options {
            if let Some(mem_aps) = opts.get_item("mem_aps") {
                let aps = mem_aps.downcast::<PyDict>()?;
                for (ap_name, ap_opts) in aps.iter() {
                    let ap_opts_dict = ap_opts.downcast::<PyDict>()?;
                    Self::add_mem_ap(
                        instance.downcast::<PyCell<Self>>()?,
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
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.arm_debug = Some(arm_debug);
        Ok(())
    }

    fn add_mem_ap(slf: &PyCell<Self>, name: &str, ap: Option<u32>, csw_reset: Option<u32>) -> PyResult<()> {
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

        slf.call_method("add_sub_block", args, Some(kwargs))?;
        Ok(())
    }

    #[args(line_reset="true")]
    fn switch_to_swd(mut slf: PyRefMut<Self>, line_reset: bool) -> PyResult<Py<Self>> {
        slf.arm_debug.as_mut().expect("Arm Debug hasn't been initialized yet!").switch_to_swd(line_reset)?;
        Ok(slf.into())
    }

    // #[args(data="None")]
    // fn verify_dp_idcode(&self, data: Option<&PyAny>) -> PyResult<()> {
    //     Ok(())
    // }

    fn power_up(slf: &PyCell<Self>) -> PyResult<()> {
        let dp = slf.getattr("dp")?.extract::<DP>()?;
        dp.power_up()
    }

    fn verify_powered_up(slf: &PyCell<Self>) -> PyResult<()> {
        let dp = slf.getattr("dp")?.extract::<DP>()?;
        dp.verify_powered_up()
    }
}

macro_rules! get_dp {
    ( $slf:expr ) => {
        $slf.dp.as_ref().expect("DP has not been initialized yet!")
    };
}

#[pyclass(subclass)]
#[text_signature = "()"]
#[derive(Clone)]
struct DP {
    pub dp: Option<OrigenDP>,
}

#[pymethods]
impl DP {
    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self {
            dp: None,
        }
    }

    #[classmethod]
    #[args(_block_options="**")]
    fn model_init(_cls: &PyType, instance: &PyAny, _block_options: Option<&PyDict>) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        let args = PyTuple::new(py, &["default".to_object(py), "default".to_object(py)]);
        let dp;
        {
            let mut dut = origen::dut();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            dp = OrigenDP::model_init(&mut dut, model_id)?;
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.dp = Some(dp);
        obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    fn write_register(&self, bits: &PyAny) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        get_dp!(self).write_register(&dut, &bc.materialize(&dut)?)?;
        Ok(())
    }

    fn verify_register(&self, bits: &PyAny) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        get_dp!(self).verify_register(&dut, &bc.materialize(&dut)?)?;
        Ok(())
    }

    fn power_up(&self) -> PyResult<()> {
        let mut dut = origen::dut();
        get_dp!(self).power_up(&mut dut)?;
        Ok(())
    }

    fn verify_powered_up(&self) -> PyResult<()> {
        let mut dut = origen::dut();
        get_dp!(self).verify_powered_up(&mut dut)?;
        Ok(())
    }
}

macro_rules! get_mem_ap {
    ( $slf:expr ) => {
        $slf.mem_ap.as_ref().expect("MemAP has not been initialized yet!")
    };
}

#[pyclass(subclass)]
struct MemAP {
    pub mem_ap: Option<OrigenMemAP>,
}

#[pymethods]
impl MemAP {

    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
            Self { 
                mem_ap: None
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
        let mem_ap;
        {
            let mut dut = origen::dut();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            mem_ap = OrigenMemAP::model_init(&mut dut, model_id, addr)?;
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.mem_ap = Some(mem_ap);
        obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    /// Initiates an ArmDebug MemAP write based on the given register (passed in as
    /// a BitCollection).
    /// Assumes that all posturing has been completed - that is, the bits' data, overlay
    /// status, etc. is current.
    fn write_register(&self, bits: &PyAny, _latency: Option<u32>) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        get_mem_ap!(self).write_register(&dut, &bc.materialize(&dut)?)?;
        Ok(())
    }
}
