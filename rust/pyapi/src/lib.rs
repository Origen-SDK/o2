#[allow(unused_imports)]
#[macro_use]
extern crate origen;
#[macro_use]
extern crate origen_metal;

use pyapi_metal;

#[macro_use]
mod macros;

mod current_command;
mod dut;
mod extensions;
mod file_handler;
mod meta;
mod model;
#[macro_use]
mod pins;
mod registers;
mod services;
#[macro_use]
mod timesets;
mod _frontend;
mod _helpers;
mod application;
mod producer;
mod prog_gen;
mod standard_sub_blocks;
mod tester;
mod tester_apis;
#[macro_use]
mod utility;
mod plugins;

use crate::registers::bit_collection::BitCollection;
use num_bigint::BigUint;
use om::lazy_static::lazy_static;
use origen::{Dut, Error, Operation, Result, Value, FLOW, ORIGEN_CONFIG, STATUS, TEST, clean_target};
use origen_metal as om;
use pyapi_metal::{runtime_error, pypath};
use pyo3::conversion::AsPyPointer;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyDict};
use pyo3::{wrap_pyfunction, wrap_pymodule};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::MutexGuard;
use utility::location::Location;
use paste::paste;

use crate::dut::__PYO3_PYMODULE_DEF_DUT;
use crate::tester::__PYO3_PYMODULE_DEF_TESTER;
use crate::tester_apis::__PYO3_PYMODULE_DEF_TESTER_APIS;
use crate::application::__PYO3_PYMODULE_DEF_APPLICATION;
use crate::prog_gen::interface::__PYO3_PYMODULE_DEF_INTERFACE;
use crate::producer::__PYO3_PYMODULE_DEF_PRODUCER;
use crate::services::__PYO3_PYMODULE_DEF_SERVICES;
use crate::utility::__PYO3_PYMODULE_DEF_UTILITY;
use crate::standard_sub_blocks::__PYO3_PYMODULE_DEF_STANDARD_SUB_BLOCKS;
use crate::prog_gen::__PYO3_PYMODULE_DEF_PROG_GEN;

use pyapi_metal::__PYO3_PYMODULE_DEF__ORIGEN_METAL;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[pymodule]
/// This is the top-level _origen module which can be imported by Python
fn _origen(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(initialize))?;
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(version))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(config_metadata))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;
    m.add_wrapped(wrap_pyfunction!(is_app_present))?;
    m.add_wrapped(wrap_pyfunction!(clean_mode))?;
    m.add_wrapped(wrap_pyfunction!(target_file))?;
    m.add_wrapped(wrap_pyfunction!(test))?;
    m.add_wrapped(wrap_pyfunction!(test_ast))?;
    m.add_wrapped(wrap_pyfunction!(flow))?;
    m.add_wrapped(wrap_pyfunction!(flow_ast))?;
    m.add_wrapped(wrap_pyfunction!(output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_source_directory))?;
    m.add_wrapped(wrap_pyfunction!(prepare_for_target_load))?;
    m.add_wrapped(wrap_pyfunction!(start_new_test))?;
    m.add_wrapped(wrap_pyfunction!(unhandled_error_count))?;
    m.add_wrapped(wrap_pyfunction!(set_output_dir))?;
    m.add_wrapped(wrap_pyfunction!(set_reference_dir))?;
    m.add_wrapped(wrap_pyfunction!(exit_pass))?;
    m.add_wrapped(wrap_pyfunction!(exit_fail))?;
    m.add_wrapped(wrap_pyfunction!(enable_debug))?;
    m.add_wrapped(wrap_pyfunction!(set_operation))?;
    m.add_wrapped(wrap_pyfunction!(boot_users))?;

    m.add_wrapped(wrap_pymodule!(dut))?;
    m.add_wrapped(wrap_pymodule!(tester))?;
    m.add_wrapped(wrap_pymodule!(application))?;
    m.add_wrapped(wrap_pymodule!(interface))?;
    m.add_wrapped(wrap_pymodule!(producer))?;
    m.add_wrapped(wrap_pymodule!(services))?;
    m.add_wrapped(wrap_pymodule!(utility))?;
    m.add_wrapped(wrap_pymodule!(tester_apis))?;
    m.add_wrapped(wrap_pymodule!(standard_sub_blocks))?;
    m.add_wrapped(wrap_pymodule!(prog_gen))?;

    file_handler::define(m)?;
    plugins::define(py, m)?;
    extensions::define(py, m)?;
    current_command::define(py, m)?;

    // Compile the _origen_metal library along with this one
    // to allow re-use from that library
    m.add_wrapped(wrap_pymodule!(_origen_metal))?;
    m.setattr(current_command::ATTR_NAME, py.None())?;
    Ok(())
}

fn extract_value<'a>(
    bits_or_val: &PyAny,
    size: Option<u32>,
    dut: &'a MutexGuard<Dut>,
) -> Result<Value<'a>> {
    let bits = bits_or_val.extract::<PyRef<BitCollection>>();
    if bits.is_ok() {
        return Ok(Value::Bits(bits.unwrap().materialize(dut)?, size));
    }
    let value = bits_or_val.extract::<BigUint>();
    if value.is_ok() {
        return match size {
            Some(x) => Ok(Value::Data(value.unwrap(), x)),
            None => Err(Error::new(
                "A size argument must be supplied along with a data value",
            )),
        };
    }
    Err(Error::new("Illegal bits/value argument"))
}

/// Unpacks/extracts common transaction options, updating the transaction directly
/// Unpacks: addr(u128), overlay (BigUint), overlay_str(String), mask(BigUint),
fn unpack_transaction_options(
    trans: &mut origen::Transaction,
    kwargs: Option<&PyDict>,
) -> PyResult<()> {
    if let Some(opts) = kwargs {
        if let Some(address) = opts.get_item("address") {
            trans.address = Some(address.extract::<BigUint>()?);
        }
        if let Some(w) = opts.get_item("address_width") {
            trans.address_width = Some(w.extract::<usize>()?);
        }
        if let Some(_mask) = opts.get_item("mask") {
            panic!("option not supported yet!");
        }
        if let Some(_overlay) = opts.get_item("overlay") {
            panic!("option not supported yet!");
        }
        if let Some(_overlay_str) = opts.get_item("overlay_str") {
            panic!("option not supported yet!");
        }
    }
    Ok(())
}

fn unpack_capture_kwargs(
    dut: &origen::Dut,
    cap_trans: &mut origen::Capture,
    kwargs: Option<&PyDict>,
    pins_allowed: bool,
    cycles_allowed: bool,
) -> PyResult<()> {
    if let Some(opts) = kwargs {
        if let Some(sym) = opts.get_item("symbol") {
            cap_trans.symbol = Some(sym.extract::<String>()?);
        }
        if let Some(enables) = opts.get_item("mask") {
            cap_trans.enables = Some(enables.extract::<BigUint>()?);
        }
        if let Some(cycles) = opts.get_item("cycles") {
            if cycles_allowed {
                cap_trans.cycles = Some(cycles.extract::<usize>()?);
            } else {
                return runtime_error!("'cycles' capture option is not valid in this context");
            }
        }
        if let Some(pins) = opts.get_item("pins") {
            if pins_allowed {
                let pins_vec = pins.extract::<Vec<&PyAny>>()?;
                cap_trans.pin_ids = Some(pins::vec_to_ppin_ids(&dut, pins_vec)?);
            } else {
                return runtime_error!("'pins' capture option is not valid in this context");
            }
        }
    }
    Ok(())
}

/// Unpacks/extracts common transaction options, updating the transaction directly
/// Unpacks: addr(u128), overlay (BigUint), overlay_str(String), mask(BigUint),
fn unpack_transaction_kwargs(trans: &mut origen::Transaction, kwargs: &PyDict) -> PyResult<()> {
    if let Some(mask) = kwargs.get_item("mask") {
        if let Ok(big_mask) = mask.extract::<num_bigint::BigUint>() {
            trans.bit_enable = big_mask;
        } else {
            return crate::type_error!("Could not extract kwarg 'mask' as an integer");
        }
    }
    if let Some(overlay) = kwargs.get_item("overlay") {
        let overlay_mask;
        let overlay_symbol;
        let overlay_cycles;
        if let Some(mask) = kwargs.get_item("overlay_mask") {
            if let Ok(big_mask) = mask.extract::<num_bigint::BigUint>() {
                overlay_mask = Some(big_mask);
            } else {
                return crate::type_error!("Could not extract kwarg 'overlay_mask' as an integer");
            }
        } else {
            if let Some(ovl) = trans.overlay.as_ref() {
                overlay_mask = ovl.enables.clone();
            } else {
                overlay_mask = None;
            }
        }
        if let Some(s) = kwargs.get_item("overlay_symbol") {
            if let Ok(sym) = s.extract::<String>() {
                overlay_symbol = Some(sym);
            } else {
                return crate::type_error!("Could not extract kwarg 'overlay_symbol' as a String");
            }
        } else {
            if let Some(ovl) = trans.overlay.as_ref() {
                overlay_symbol = ovl.symbol.clone();
            } else {
                overlay_symbol = None;
            }
        }
        if let Some(c) = kwargs.get_item("overlay_cycles") {
            if let Ok(i) = c.extract::<usize>() {
                overlay_cycles = Some(i);
            } else {
                return crate::type_error!(
                    "Could not extract kwarg 'overlay_cycles' as an Integer"
                );
            }
        } else {
            if let Some(ovl) = trans.overlay.as_ref() {
                overlay_cycles = ovl.cycles.clone();
            } else {
                overlay_cycles = None;
            }
        }
        if let Ok(should_overlay) = overlay.extract::<bool>() {
            if should_overlay {
                // Unnamed overlay
                trans.apply_overlay(None, overlay_symbol, overlay_mask)?;
            }
        } else if let Ok(overlay_name) = overlay.extract::<String>() {
            trans.apply_overlay(Some(overlay_name), overlay_symbol, overlay_mask)?;
            if overlay_cycles.is_some() {
                trans.overlay.as_mut().unwrap().cycles = overlay_cycles;
            }
        } else {
            return crate::type_error!(
                "Could not extract kwarg 'overlay' as either a bool or a string"
            );
        }
    }
    Ok(())
}

// fn unpack_register_transaction() -> PyResult<Transaction> {
//     // ...
// }

fn resolve_transaction(
    dut: &std::sync::MutexGuard<origen::Dut>,
    trans: &PyAny,
    action: Option<origen::TransactionAction>,
    kwargs: Option<&PyDict>,
) -> PyResult<origen::Transaction> {
    let mut width = 32;
    if let Some(opts) = kwargs {
        if let Some(w) = opts.get_item("width") {
            width = w.extract::<u32>()?;
        }
    }
    let value = extract_value(trans, Some(width), &dut)?;
    let mut trans;
    if let Some(a) = action {
        match a {
            origen::TransactionAction::Write => trans = value.to_write_transaction(&dut)?,
            origen::TransactionAction::Verify => trans = value.to_verify_transaction(&dut)?,
            origen::TransactionAction::Capture => {
                trans = value.to_capture_transaction(&dut)?;
                unpack_capture_kwargs(
                    &dut,
                    &mut trans.capture.as_mut().unwrap(),
                    kwargs,
                    false,
                    false,
                )?;
                return Ok(trans);
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Resolving transactions for {:?} is not supported",
                    a
                )))
            }
        }
    } else {
        trans = value.to_write_transaction(&dut)?;
        trans.action = None;
    }

    if let Some(opts) = kwargs {
        if let Some(address) = opts.get_item("address") {
            if !address.is_none() {
                trans.address = Some(address.extract::<BigUint>()?);
            }
        }
        if let Some(w) = opts.get_item("address_width") {
            trans.address_width = Some(w.extract::<usize>()?);
        }
        if let Some(_mask) = opts.get_item("mask") {
            panic!("option not supported yet!");
        }
        if let Some(_overlay) = opts.get_item("overlay") {
            panic!("option not supported yet!");
        }
        if let Some(_overlay_str) = opts.get_item("overlay_str") {
            panic!("option not supported yet!");
        }
    }
    Ok(trans)
}

/// Exit with a failing status code and print a big FAIL to the console
#[pyfunction]
fn exit_fail() -> PyResult<()> {
    exit_fail!();
}

/// Exit with a passing status code and print a big PASS to the console
#[pyfunction]
fn exit_pass() -> PyResult<()> {
    exit_pass!();
}

fn origen_mod_path() -> PyResult<PathBuf> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item("importlib", py.import("importlib")?)?;
    let p = PathBuf::from(
        py.eval(
            "importlib.util.find_spec('_origen').origin",
            None,
            Some(&locals),
        )?
        .extract::<String>()?,
    );
    Ok(p.parent().unwrap().to_path_buf())
}

/// Called automatically when Origen is first loaded
#[pyfunction]
fn initialize(
    py: Python,
    log_verbosity: Option<u8>,
    verbosity_keywords: Vec<String>,
    cli_location: Option<String>,
    cli_version: Option<String>,
    fe_pkg_loc: Option<PathBuf>,
    fe_exe_loc: Option<PathBuf>,
) -> PyResult<()> {
    origen::initialize(log_verbosity, verbosity_keywords, cli_location, cli_version, fe_pkg_loc, fe_exe_loc);
    origen::STATUS.update_other_build_info("pyapi_version", built_info::PKG_VERSION)?;
    origen::FRONTEND
        .write()
        .unwrap()
        .set_frontend(Box::new(_frontend::Frontend::new()))?;

    if let Some(app) = &STATUS.app {
        origen::STATUS.set_in_origen_core_app(origen_mod_path()? == app.root);
    } else {
        origen::STATUS.set_in_origen_core_app(false);
    }

    use crate::pyapi_metal::prelude::frontend::*;
    with_mut_py_data_stores(|py, mut py_ds| {
        // TODO remove some of this hardcoding
        let pymod = PyModule::import(py, "_origen")?;
        py_ds.add_category(
            py,
            "ldaps",
            Some(pymod.getattr("utility")?.getattr("boot_ldaps")?.into()),
            Some(true),
        )?;
        Ok(())
    })?;
    boot_users(py)?;
    match origen::setup_sessions() {
        Ok(_) => {}
        Err(e) => log_error!(
            "Failed to setup user and application sessions. Received error: \n{}",
            e
        ),
    }
    Ok(())
}

#[pyfunction]
/// Set the output directory to be used instead of <APP ROOT>/output
fn set_output_dir(dir: &str) -> PyResult<()> {
    STATUS.set_output_dir(Path::new(dir));
    Ok(())
}

#[pyfunction]
/// Set the output directory to be used instead of <APP ROOT>/output
fn set_reference_dir(dir: &str) -> PyResult<()> {
    STATUS.set_reference_dir(Path::new(dir));
    Ok(())
}

#[pyfunction]
/// Enable Python source line tracking
fn enable_debug() -> PyResult<()> {
    STATUS.set_debug_enabled(true);
    Ok(())
}

#[pyfunction]
/// Set the current Origen operation (generate, compile, etc.)
fn set_operation(name: String) -> PyResult<()> {
    match Operation::from_str(&name) {
        Ok(op) => {
            STATUS.set_operation(op);
            Ok(())
        }
        Err(e) => Err(PyErr::from(Error::new(&e))),
    }
}

#[pyfunction]
/// Returns the number of unhandled errors that have been encountered since the Origen
/// invocation started.
/// An unhandled error is something that ultimately resulted in a pattern not being generated
/// or something equally serious.
fn unhandled_error_count() -> PyResult<usize> {
    Ok(STATUS.unhandled_error_count())
}

/// Prints out the AST for the current test to the console
#[pyfunction]
fn test() -> PyResult<()> {
    println!("{}", TEST.to_string());
    Ok(())
}

/// Returns the AST for the current test in Python
#[pyfunction]
fn test_ast() -> PyResult<Vec<u8>> {
    Ok(TEST.to_pickle())
}

/// Prints out the AST for the current flow to the console
#[pyfunction]
fn flow() -> PyResult<()> {
    println!("{}", FLOW.to_string());
    Ok(())
}

/// Returns the AST for the current flow in Python
#[pyfunction]
fn flow_ast() -> PyResult<Vec<u8>> {
    Ok(FLOW.to_pickle())
}

/// Returns the Origen status which informs whether an app is present, the Origen version,
/// etc.
#[pyfunction]
fn status(py: Python) -> PyResult<PyObject> {
    let ret = PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("is_app_present", &STATUS.is_app_present);
    if let Some(app) = origen::app() {
        let _ = ret.set_item("root", format!("{}", app.root.display()));
    }
    let _ = ret.set_item("origen_version", &STATUS.origen_version.to_string());
    let _ = ret.set_item("home", format!("{}", STATUS.home.display()));
    let _ = ret.set_item("on_windows", om::running_on_windows());
    ret.set_item(
        "origen_core_support_version",
        STATUS.origen_core_support_version.to_string(),
    )?;
    ret.set_item(
        "origen_metal_backend_version",
        STATUS.origen_metal_backend_version.to_string(),
    )?;
    ret.set_item(
        "other_build_info",
        pyapi_metal::_helpers::map_to_pydict(py, &mut STATUS.other_build_info().iter())?,
    )?;
    ret.set_item(
        "cli_version",
        match STATUS.cli_version() {
            Some(v) => Some(v.to_string()).to_object(py),
            None => py.None(),
        },
    )?;
    ret.set_item(
        "is_app_in_origen_dev_mode",
        STATUS.is_app_in_origen_dev_mode,
    )?;
    ret.set_item("in_origen_core_app", STATUS.in_origen_core_app())?;
    Ok(ret.into())
}

/// Returns the Origen version formatted into PEP440, e.g. "1.2.3.dev4"
#[pyfunction]
fn version() -> PyResult<String> {
    Ok(
        origen::utility::version::Version::new_pep440(&STATUS.origen_version.to_string())?
            .to_string(),
    )
}

/// Returns the Origen configuration (as defined in origen.toml files)
#[pyfunction]
fn config(py: Python) -> PyResult<PyObject> {
    let ret = PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("python_cmd", &ORIGEN_CONFIG.python_cmd);
    let _ = ret.set_item("pkg_server", &ORIGEN_CONFIG.pkg_server);
    let _ = ret.set_item("pkg_server_push", &ORIGEN_CONFIG.pkg_server_push);
    let _ = ret.set_item("pkg_server_pull", &ORIGEN_CONFIG.pkg_server_pull);
    let _ = ret.set_item("some_val", &ORIGEN_CONFIG.some_val);
    Ok(ret.into())
}

#[pyfunction]
fn config_metadata<'py>(py: Python<'py>) -> PyResult<&'py PyDict> {
    let m = origen::origen_config_metadata();
    let retn = PyDict::new(py);
    retn.set_item("files", m.files.iter().map( |p| Ok(pypath!(py, p.display()))).collect::<PyResult<Vec<PyObject>>>()?)?;
    Ok(retn)
}

#[pyfunction]
fn is_app_present() -> PyResult<bool> {
    Ok(STATUS.is_app_present)
}

/// Returns the Origen application configuration (as defined in application.toml)
#[pyfunction]
fn app_config(py: Python) -> PyResult<Option<PyObject>> {
    if let Some(app) = origen::app() {
        let ret = PyDict::new(py);
        let _ = app.with_config(|config| {
            let _ = ret.set_item("name", &config.name);
            let _ = ret.set_item("target", &config.target);
            let _ = ret.set_item("mode", &config.mode);
            let _ = ret.set_item("__output_directory__", &config.output_directory);
            let _ = ret.set_item(
                "__website_output_directory__",
                &config.website_output_directory,
            );
            let _ = ret.set_item(
                "__website_source_directory__",
                &config.website_source_directory,
            );
            let _ = ret.set_item(
                "website_release_location",
                match &config.website_release_location {
                    Some(loc) => Py::new(
                        py,
                        Location {
                            location: (*loc).clone(),
                        },
                    )
                    .unwrap()
                    .to_object(py),
                    None => py.None(),
                },
            );
            let _ = ret.set_item("website_release_name", &config.website_release_name);
            Ok(())
        });
        Ok(Some(ret.into()))
    } else {
        Ok(None)
    }
}

/// clean_mode(name)
/// Sanitizes the given mode string and returns it, but will exit the process if it is invalid
#[pyfunction]
fn clean_mode(name: &str) -> PyResult<String> {
    let c = origen::clean_mode(name);
    Ok(c)
}

#[pyfunction]
/// target_file(name, dir)
/// Sanitizes the given target/env name and returns the matching file, but will exit the process
/// if it does not uniquely identify a single target/env file.
fn target_file(name: &str, dir: &str) -> PyResult<String> {
    let c = clean_target!(name, dir, true);
    Ok(c)
}

#[pyfunction]
fn output_directory(py: Python) -> PyResult<PyObject> {
    let dir = origen::STATUS.output_dir();
    Ok(pypath!(py, dir.display()))
}

#[pyfunction]
fn website_output_directory(py: Python) -> PyResult<PyObject> {
    let dir = origen::app().unwrap().website_output_directory();
    Ok(pypath!(py, dir.display()))
}

#[pyfunction]
fn website_source_directory(py: Python) -> PyResult<PyObject> {
    let dir = origen::app().unwrap().website_source_directory();
    Ok(pypath!(py, dir.display()))
}

#[pyfunction]
/// This will be called by Origen immediately before loading a fresh set of targets
fn prepare_for_target_load() -> PyResult<()> {
    origen::prepare_for_target_load()?;
    Ok(())
}

#[pyfunction]
/// Clears the current test (pattern) AST and starts a new one
fn start_new_test(name: Option<String>) -> PyResult<()> {
    origen::start_new_test(name);
    Ok(())
}

pub fn pickle(py: Python, object: &impl AsPyPointer) -> PyResult<Vec<u8>> {
    let pickle = PyModule::import(py, "pickle")?;
    pickle
        .getattr("dumps")?
        .call1((object,))?
        .extract::<Vec<u8>>()
}

pub fn depickle<'a>(py: Python<'a>, object: &Vec<u8>) -> PyResult<&'a PyAny> {
    let pickle = PyModule::import(py, "pickle")?;
    let bytes = PyBytes::new(py, object);
    pickle.getattr("loads")?.call1((bytes,))
}

pub fn with_pycallbacks<T, F>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, &PyAny) -> PyResult<T>,
{
    let gil = Python::acquire_gil();
    let py = gil.python();

    let pycallbacks = py.import("origen.callbacks")?;
    func(py, pycallbacks)
}

pub fn get_full_class_name(obj: &PyAny) -> PyResult<String> {
    let cls = obj.getattr("__class__")?;
    let mut n = cls.getattr("__module__")?.extract::<String>()?;
    n.push_str(&format!(
        ".{}",
        cls.getattr("__qualname__")?.extract::<String>()?
    ));
    Ok(n)
}

// TODO probably move this to somewhere else
#[pyfunction]
pub fn boot_users(py: Python) -> PyResult<pyapi_metal::framework::users::Users> {
    lazy_static! {
        static ref BASE_MSG: &'static str = "Encountered an error when initializing users";
    }

    log_trace!("Initializing Users...");
    if let Some(r) = &ORIGEN_CONFIG.session__user_root {
        log_trace!("Setting user session root to {}", r);
        let mut users = om::users_mut();
        let mut sc = users.default_session_config_mut();
        sc.root = Some(PathBuf::from(r));
    }

    let users = pyapi_metal::framework::users::users()?;

    if let Some(pw_cache_option) = &crate::ORIGEN_CONFIG.user__password_cache_option {
        match users.set_default_password_cache_option(Some(pw_cache_option)) {
            Ok(_) => {},
            Err(e) => {
                om::log_error!("{}: Error encountered updating default password cache option", *BASE_MSG);
                om::log_error!("{}", e);
            }
        }
    }

    if let Some(dsets) = &crate::ORIGEN_CONFIG.user__datasets {
        let mut replace_default = true;
        for (dn, config) in dsets {
            log_trace!("Adding user dataset {}", dn);
            match config.try_into() {
                Ok(om_config) => {
                    match pyapi_metal::framework::users::UserDatasetConfig::new_py(py, om_config) {
                        Ok(py_config) => {
                            if replace_default {
                                match users.override_default_dataset(
                                    dn,
                                    Some(py_config.into_py(py).as_ref(py)),
                                ) {
                                    Ok(_) => {
                                        replace_default = false;
                                    }
                                    Err(e) => {
                                        om::log_error!("{}: Error encountered updating default dataset with config '{}'", *BASE_MSG, dn);
                                        om::log_error!("{}", e);
                                    }
                                }
                            } else {
                                match users.add_dataset(
                                    dn,
                                    Some(
                                        pyapi_metal::framework::users::UserDatasetConfig::new_py(
                                            py,
                                            config.try_into()?,
                                        )?
                                        .into_py(py)
                                        .as_ref(py),
                                    ),
                                    false,
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        om::log_error!(
                                            "{}: Error encountered adding dataset '{}'",
                                            *BASE_MSG,
                                            dn
                                        );
                                        om::log_error!("{}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Still in the "processing stage - just on the python side
                            om::log_error!(
                                "{}: Error encountered processing dataset config for '{}'",
                                *BASE_MSG,
                                dn
                            );
                            om::log_error!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    om::log_error!(
                        "{}: Error encountered processing dataset config for '{}'",
                        *BASE_MSG,
                        dn
                    );
                    om::log_error!("{}", e);
                }
            }
        }
    }

    // Set the data lookup hierarchy
    if let Some(hierarchy) = &crate::ORIGEN_CONFIG.user__data_lookup_hierarchy {
        match users.set_data_lookup_hierarchy(hierarchy.to_owned()) {
            Ok(_) => {}
            Err(e) => {
                om::log_error!(
                    "{}: Error encountered setting the default lookup hierarchy",
                    *BASE_MSG
                );
                om::log_error!("{}", e);
                om::log_error!("Forcing empty dataset lookup hierarchy...");
                users.set_data_lookup_hierarchy(vec![])?;
            }
        }
    } else {
        if crate::ORIGEN_CONFIG.user__datasets.is_some()
            && crate::ORIGEN_CONFIG.user__datasets.as_ref().unwrap().len() > 1
        {
            // The config can only be read as an unordered hashmap. If multiple datasets are given,
            // clear the hierarchy if not explicitly given, otherwise will get non-deterministic behavior
            users.set_data_lookup_hierarchy(vec![])?;
        }
    }

    // Add dataset motives
    for (m, ds) in &crate::ORIGEN_CONFIG.user__dataset_motives {
        match users.add_motive(m, ds, false) {
            Ok(_) => {}
            Err(e) => {
                om::log_error!(
                    "{}: Error encountered adding dataset motive '{}'",
                    *BASE_MSG,
                    m
                );
                om::log_error!("{}", e);
            }
        }
    }

    macro_rules! log_error__set_field_for_default_user {
        ($u:expr, $field:tt, $val:expr, $name:expr ) => {
            paste! {
                match $u.[<set_ $field>]($val) {
                    Ok(_) => {},
                    Err(e) => {
                        log_error!("{}: Failed to initialize default user '{}'", *BASE_MSG, $name);
                        log_error!("    Failed to set field '{}'", stringify!($field));
                        log_error!("{}", e);
                        log_error!("Bailing on initializing default user '{}'", $name);
                        continue;
                    }
                }
            }
        }
    }

    // Add any default users
    for (name, config) in &crate::ORIGEN_CONFIG.default_users {
        match users.add(name, config.auto_populate) {
            Ok(u) => {
                if let Some(s) = config.should_validate_passwords {
                    log_error__set_field_for_default_user!(u, should_validate_passwords, Some(s), name)
                }
                if let Some(uname) = &config.username {
                    log_error__set_field_for_default_user!(u, username, Some(uname.to_owned()), name);
                }
                if let Some(pw) = &config.password {
                    log_error__set_field_for_default_user!(u, password, Some(pw.to_owned()), name);
                }

                if let Some(e) = &config.email {
                    log_error__set_field_for_default_user!(u, email, Some(e.to_owned()), name);
                }
                if let Some(f) = &config.first_name {
                    log_error__set_field_for_default_user!(u, first_name, Some(f.to_owned()), name);
                }
                if let Some(l) = &config.last_name {
                    log_error__set_field_for_default_user!(u, last_name, Some(l.to_owned()), name);
                }
            },
            Err(e) => {
                om::log_error!("{}: Failed to initialize default user '{}'", *BASE_MSG, name);
                log_error!("{}", e);
            }
        }
    }

    // See if the frontend provides a specific means to lookup the current user
    if let Some(func) = &crate::ORIGEN_CONFIG.user__current_user_lookup_function {
        users.set_lookup_current_id_function(Some(pyapi_metal::_helpers::get_qualified_attr(&func)?.as_ref(py)))?;
    }

    // Initialize the current user
    if ORIGEN_CONFIG.initial_user.as_ref().map_or(true, |u| u.initialize.unwrap_or(true)) {
        match users.lookup_current_id(true) {
            Ok(_) => {
                if ORIGEN_CONFIG.initial_user.as_ref().map_or(true, |u| u.init_home_dir.unwrap_or(true)) {
                    match users.current_user() {
                        Ok(usr) => {
                            match usr {
                                Some(u) => {
                                    match u.set_home_dir(None) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            log_error!("{}: Failed to lookup current user's home directory", *BASE_MSG);
                                            log_error!("{}", e);
                                        }
                                    }
                                },
                                None => {
                                    log_error!("{}: Failed to lookup current user", *BASE_MSG);
                                }
                            }
                        },
                        Err(e) => {
                            log_error!("{}: Failed to lookup current user", *BASE_MSG);
                            log_error!("{}", e);
                        }
                    }
                }
            },
            Err(e) => {
                log_error!("{}: Failed to lookup current user", *BASE_MSG);
                log_error!("{}", e);
            }
        }
    } else {
        log_trace!("Bypassing current user initialization.");
    }
    Ok(users)
}
