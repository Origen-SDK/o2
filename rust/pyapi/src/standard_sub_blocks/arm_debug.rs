// use origen::standard_sub_blocks::ArmDebug as OrigenArmDebug;
use origen::services::arm_debug::ArmDebug as OrigenArmDebug;
// use origen::standard_sub_blocks::arm_debug::DP as OrigenDP;
use origen::services::arm_debug::JtagDP as OrigenJtagDP;
use origen::services::arm_debug::MemAP as OrigenMemAP;
use origen::services::arm_debug::DP as OrigenDP;
// use origen::standard_sub_blocks::arm_debug::mem_ap::MemAP as OrigenMemAP;
use crate::registers::bit_collection::BitCollection;
use crate::{extract_value, resolve_transaction, unpack_transaction_options};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple, PyType};
use pyo3::ToPyObject;

#[pymodule]
/// Implements the module _origen.standard_sub_blocks in Python and ties together
/// the PyAPI with the Rust backend.
/// Put another way, this is the Python-side controller for the backend-side model/controller.
pub fn arm_debug(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArmDebug>()?;
    m.add_class::<DP>()?;
    m.add_class::<JtagDP>()?;
    m.add_class::<MemAP>()?;
    Ok(())
}

/// Checks for an SWD attribute on the DUT
/// Note: this must be run after the DUT has loaded or else it'll cause a lockup
fn check_for_swd() -> PyResult<Option<usize>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item("origen", py.import("origen")?.to_object(py))?;
    locals.set_item("builtins", py.import("builtins")?.to_object(py))?;
    locals.set_item("dut", py.eval("origen.dut", Some(locals.clone()), None)?)?;
    let m = py.eval("builtins.hasattr(dut, \"swd\")", Some(locals), None)?;

    if m.extract::<bool>()? {
        let pyswd = py.eval("dut.swd", Some(locals), None)?;
        if let Ok(swd) = pyswd.extract::<PyRefMut<super::super::services::swd::SWD>>() {
            Ok(Some(swd.id()?))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn check_for_jtag() -> PyResult<Option<usize>> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item("origen", py.import("origen")?.to_object(py))?;
    locals.set_item("builtins", py.import("builtins")?.to_object(py))?;
    locals.set_item("dut", py.eval("origen.dut", Some(locals.clone()), None)?)?;
    let m = py.eval("builtins.hasattr(dut, \"jtag\")", Some(locals), None)?;

    if m.extract::<bool>()? {
        let pyjtag = py.eval("dut.jtag", Some(locals), None)?;
        if let Ok(jtag) = pyjtag.extract::<PyRefMut<super::super::services::jtag::JTAG>>() {
            Ok(Some(jtag.id()?))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[pyclass(subclass)]
#[derive(Clone)]
/// Controller connecting the :class:`origen.blocks.arm_debug.controller.Controller` view
/// with the :link-to:`backend model <backend_arm_debug_model>`.
/// The controller here is responsible for instantiating and initializing the
/// ArmDebug model.
pub struct ArmDebug {
    arm_debug_id: Option<usize>,
}

#[pymethods]
impl ArmDebug {
    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { arm_debug_id: None }
    }

    #[classmethod]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        crate::dut::PyDUT::ensure_pins("dut")?;
        let swd_id = check_for_swd()?;
        let jtag_id = check_for_jtag()?;

        // Create the Arm Debug instance
        let gil = Python::acquire_gil();
        let py = gil.python();
        let arm_debug_id;
        {
            let model_id = instance.getattr("model_id")?.extract::<usize>()?;
            let mut dut = origen::dut();
            let mut services = origen::services();
            arm_debug_id =
                OrigenArmDebug::model_init(&mut dut, &mut services, model_id, swd_id, jtag_id)?;
        }

        // Add the DP subblock
        let args = PyTuple::new(
            py,
            &["dp".to_object(py), "origen.arm_debug.dp".to_object(py)],
        );
        let kwargs = PyDict::new(py);
        let sb_options = PyDict::new(py);
        sb_options.set_item("arm_debug_id", arm_debug_id)?;
        kwargs.set_item("sb_options", sb_options.to_object(py))?;
        let py_dp_obj = instance.downcast::<PyCell<Self>>()?.call_method(
            "add_sub_block",
            args,
            Some(kwargs),
        )?;
        let py_dp = py_dp_obj.extract::<DP>()?;
        let dp_id = py_dp.dp_id.unwrap();

        // Now go back and add the DP ID to arm debug
        {
            let mut services = origen::services();
            let origen_arm_debug = services.get_as_mut_arm_debug(arm_debug_id)?;
            origen_arm_debug.set_dp_id(dp_id)?;
        }

        // If a jtag protocol was provided, add the JTAG DP
        if jtag_id.is_some() {
            let args = PyTuple::new(
                py,
                &[
                    "jtag_dp".to_object(py),
                    "origen.arm_debug.jtag_dp".to_object(py),
                ],
            );
            let kwargs = PyDict::new(py);
            let sb_options = PyDict::new(py);
            sb_options.set_item("arm_debug_id", arm_debug_id)?;
            kwargs.set_item("sb_options", sb_options.to_object(py))?;
            let py_jtag_dp_obj = instance.downcast::<PyCell<Self>>()?.call_method(
                "add_sub_block",
                args,
                Some(kwargs),
            )?;
            let py_jtag_dp = py_jtag_dp_obj.extract::<JtagDP>()?;
            let jtag_dp_id = py_jtag_dp.id.unwrap();

            let mut services = origen::services();
            let origen_arm_debug = services.get_as_mut_arm_debug(arm_debug_id)?;
            origen_arm_debug.set_jtag_dp_id(jtag_dp_id)?;
        }

        {
            let mut slf = instance.extract::<PyRefMut<Self>>()?;
            slf.arm_debug_id = Some(arm_debug_id);
        }
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
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    fn add_mem_ap(
        slf: &PyCell<Self>,
        name: &str,
        ap: Option<u32>,
        csw_reset: Option<u32>,
    ) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let args = PyTuple::new(
            py,
            &[name.to_object(py), "origen.arm_debug.mem_ap".to_object(py)],
        );
        let kwargs = PyDict::new(py);
        let sb_options = PyDict::new(py);
        if let Some(_ap) = ap {
            sb_options.set_item("ap", _ap)?;
        }
        if let Some(_csw_reset) = csw_reset {
            sb_options.set_item("csw_reset", _csw_reset)?;
        }
        let arm_debug = slf.extract::<PyRefMut<Self>>()?;
        sb_options.set_item("arm_debug_id", arm_debug.arm_debug_id)?;
        kwargs.set_item("sb_options", sb_options.to_object(py))?;

        slf.call_method("add_sub_block", args, Some(kwargs))?;
        Ok(())
    }

    fn switch_to_swd(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        let services = origen::services();
        let arm_debug = services.get_as_arm_debug(slf.arm_debug_id.unwrap())?;
        let dut = origen::dut();
        arm_debug.switch_to_swd(&dut, &services)?;
        Ok(slf.into())
    }
}

#[pyclass(subclass)]
#[pyo3(text_signature = "()")]
#[derive(Clone)]
struct DP {
    pub dp_id: Option<usize>,
    #[allow(dead_code)]
    pub arm_debug_id: Option<usize>,
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
            dp_id: None,
            arm_debug_id: None,
        }
    }

    #[classmethod]
    #[args(_block_options = "**")]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        // Require an ArmDebug ID to tie this DP to an ArmDebug instance
        let arm_debug_id;
        if let Some(opts) = block_options {
            if let Some(ad_id) = opts.get_item("arm_debug_id") {
                if let Ok(id) = ad_id.extract::<usize>() {
                    arm_debug_id = id;
                } else {
                    return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                        "Subblock arm_debug.dp was given an arm_debug _id block option but could not extract it as an integer"
                    ));
                }
            } else {
                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                    "Subblock arm_debug.dp was not given required block option 'arm_debug_id'",
                ));
            }
        } else {
            return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Subblock arm_debug.dp requires an arm_debug_id block option, but no block options were given."
            ));
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        let args = PyTuple::new(py, &["default".to_object(py), "default".to_object(py)]);
        let dp_id;
        {
            let mut dut = origen::dut();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            let mut services = origen::services();
            dp_id = OrigenDP::model_init(&mut dut, &mut services, model_id, arm_debug_id)?;
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.dp_id = Some(dp_id);
        obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    #[args(write_opts = "**")]
    fn write_register(&self, bits: &PyAny, _write_opts: Option<&PyDict>) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        let services = origen::services();
        let dp = services.get_as_dp(self.dp_id.unwrap())?;
        dp.write_register(&dut, &services, &bc.materialize(&dut)?)?;
        Ok(())
    }

    #[args(verify_opts = "**")]
    fn verify_register(&self, bits: &PyAny, _verify_opts: Option<&PyDict>) -> PyResult<()> {
        let bc = bits.extract::<PyRef<BitCollection>>()?;
        let dut = origen::dut();
        let services = origen::services();
        let dp = services.get_as_dp(self.dp_id.unwrap())?;
        dp.verify_register(&dut, &services, &bc.materialize(&dut)?)?;
        Ok(())
    }

    fn power_up(&self) -> PyResult<()> {
        let mut dut = origen::dut();
        let services = origen::services();
        let dp = services.get_as_dp(self.dp_id.unwrap())?;
        dp.power_up(&mut dut, &services)?;
        Ok(())
    }

    fn verify_powered_up(&self) -> PyResult<()> {
        let mut dut = origen::dut();
        let services = origen::services();
        let dp = services.get_as_dp(self.dp_id.unwrap())?;
        dp.verify_powered_up(&mut dut, &services)?;
        Ok(())
    }
}

#[pyclass(subclass)]
#[pyo3(text_signature = "()")]
#[derive(Clone)]
struct JtagDP {
    pub id: Option<usize>,
    #[allow(dead_code)]
    pub arm_debug_id: Option<usize>,
}

#[pymethods]
impl JtagDP {
    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self {
            id: None,
            arm_debug_id: None,
        }
    }

    #[classmethod]
    #[args(_block_options = "**")]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        // Require an ArmDebug ID to tie this DP to an ArmDebug instance
        let arm_debug_id;
        if let Some(opts) = block_options {
            if let Some(ad_id) = opts.get_item("arm_debug_id") {
                if let Ok(id) = ad_id.extract::<usize>() {
                    arm_debug_id = id;
                } else {
                    return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                        "Subblock arm_debug.dp was given an arm_debug _id block option but could not extract it as an integer"
                    ));
                }
            } else {
                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                    "Subblock arm_debug.dp was not given required block option 'arm_debug_id'",
                ));
            }
        } else {
            return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Subblock arm_debug.dp requires an arm_debug_id block option, but no block options were given."
            ));
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        let args = PyTuple::new(py, &["default".to_object(py), "default".to_object(py)]);
        let id;
        {
            let mut dut = origen::dut();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            let mut services = origen::services();
            id = OrigenJtagDP::model_init(
                &mut dut,
                &mut services,
                model_id,
                arm_debug_id,
                {
                    if let Some(opts) = block_options {
                        if let Some(default_ir_size) = opts.get_item("default_ir_size") {
                            if let Ok(_default_ir_size) = default_ir_size.extract::<usize>() {
                                Some(_default_ir_size)
                            } else {
                                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                                    "Subblock arm_debug.jtag_dp was given a 'default_ifr_size' block option but could not extract it as an integer"
                                ));
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                {
                    if let Some(opts) = block_options {
                        if let Some(default_idcode) = opts.get_item("default_idcode") {
                            if let Ok(_default_idcode) = default_idcode.extract::<u32>() {
                                Some(_default_idcode)
                            } else {
                                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                                    "Subblock arm_debug.jtag_dp was given a 'default_idcode' block option but could not extract it as an integer"
                                ));
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                {
                    if let Some(opts) = block_options {
                        if let Some(dpacc_select) = opts.get_item("dpacc_select") {
                            if let Ok(_dpacc_select) = dpacc_select.extract::<u32>() {
                                Some(_dpacc_select)
                            } else {
                                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                                    "Subblock arm_debug.jtag_dp was given a 'dpacc_select' block option but could not extract it as an integer"
                                ));
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                {
                    if let Some(opts) = block_options {
                        if let Some(apacc_select) = opts.get_item("apacc_select") {
                            if let Ok(_apacc_select) = apacc_select.extract::<u32>() {
                                Some(_apacc_select)
                            } else {
                                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                                    "Subblock arm_debug.jtag_dp was given a 'apacc_select' block option but could not extract it as an integer"
                                ));
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
            )?;
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.id = Some(id);
        obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    #[args(write_opts = "**")]
    fn write_register(&self, bits: &PyAny, write_opts: Option<&PyDict>) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let jtag_dp = services.get_as_jtag_dp(self.id.unwrap())?;

        let value = extract_value(bits, Some(32), &dut)?;
        let mut trans = value.to_write_transaction(&dut)?;
        unpack_transaction_options(&mut trans, write_opts)?;
        jtag_dp.write_register(&dut, &services, trans)?;
        Ok(())
    }

    #[args(verify_opts = "**")]
    fn verify_register(&self, bits: &PyAny, verify_opts: Option<&PyDict>) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let jtag_dp = services.get_as_jtag_dp(self.id.unwrap())?;

        let value = extract_value(bits, Some(32), &dut)?;
        let mut trans = value.to_verify_transaction(&dut)?;
        unpack_transaction_options(&mut trans, verify_opts)?;
        jtag_dp.verify_register(&dut, &services, trans)?;
        Ok(())
    }
}

#[pyclass(subclass)]
struct MemAP {
    pub mem_ap_id: Option<usize>,
}

#[pymethods]
impl MemAP {
    #[classmethod]
    fn __init__(_cls: &PyType, _instance: &PyAny) -> PyResult<()> {
        Ok(())
    }

    #[new]
    fn new() -> Self {
        Self { mem_ap_id: None }
    }

    #[classmethod]
    fn model_init(_cls: &PyType, instance: &PyAny, block_options: Option<&PyDict>) -> PyResult<()> {
        // Require an ArmDebug ID to tie this DP to an ArmDebug instance
        let arm_debug_id;
        if let Some(opts) = block_options {
            if let Some(ad_id) = opts.get_item("arm_debug_id") {
                if let Ok(id) = ad_id.extract::<usize>() {
                    arm_debug_id = id;
                } else {
                    return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                        "Subblock arm_debug.mem_ap was given an arm_debug _id block option but could not extract it as an integer"
                    ));
                }
            } else {
                return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                    "Subblock arm_debug.mem_ap was not given required block option 'arm_debug_id'",
                ));
            }
        } else {
            return Err(PyErr::new::<exceptions::PyRuntimeError, _>(
                "Subblock arm_debug.mem_ap requires an arm_debug_id block option, but no block options were given."
            ));
        }

        let addr;
        //let csw_reset;
        if let Some(ap_opts_dict) = block_options {
            if let Some(ap_addr) = ap_opts_dict.get_item("ap") {
                addr = ap_addr.extract::<usize>()?;
            } else {
                addr = 0;
            }
        } else {
            addr = 0;
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = instance.to_object(py);
        let args = PyTuple::new(py, &["default".to_object(py), "default".to_object(py)]);
        let mem_ap_id;
        {
            let mut dut = origen::dut();
            let mut services = origen::services();
            let model_id = obj.getattr(py, "model_id")?.extract::<usize>(py)?;
            mem_ap_id =
                OrigenMemAP::model_init(&mut dut, &mut services, model_id, arm_debug_id, addr)?;
        }
        let mut slf = instance.extract::<PyRefMut<Self>>()?;
        slf.mem_ap_id = Some(mem_ap_id);
        obj.call_method1(py, "_set_as_default_address_block", args)?;
        Ok(())
    }

    /// Initiates an ArmDebug MemAP write based on the given register (passed in as
    /// a BitCollection).
    /// Assumes that all posturing has been completed - that is, the bits' data, overlay
    /// status, etc. is current.
    #[args(write_opts = "**")]
    fn write_register(&self, bits: &PyAny, write_opts: Option<&PyDict>) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let ap = services.get_as_mem_ap(self.mem_ap_id.unwrap())?;
        let trans = resolve_transaction(
            &dut,
            bits,
            Some(origen::TransactionAction::Write),
            write_opts,
        )?;
        ap.write_register(&dut, &services, &trans)?;
        Ok(())
    }

    #[args(verify_opts = "**")]
    fn verify_register(&self, bits: &PyAny, verify_opts: Option<&PyDict>) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let ap = services.get_as_mem_ap(self.mem_ap_id.unwrap())?;
        let trans = resolve_transaction(
            &dut,
            bits,
            Some(origen::TransactionAction::Verify),
            verify_opts,
        )?;
        ap.verify_register(&dut, &services, &trans)?;
        Ok(())
    }

    #[args(capture_opts = "**")]
    fn capture_register(&self, bits: &PyAny, capture_opts: Option<&PyDict>) -> PyResult<()> {
        let dut = origen::dut();
        let services = origen::services();
        let ap = services.get_as_mem_ap(self.mem_ap_id.unwrap())?;
        let trans = resolve_transaction(
            &dut,
            bits,
            Some(origen::TransactionAction::Capture),
            capture_opts,
        )?;
        ap.verify_register(&dut, &services, &trans)?;
        Ok(())
    }
}
