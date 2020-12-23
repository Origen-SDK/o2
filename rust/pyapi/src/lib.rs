#[allow(unused_imports)]
#[macro_use]
extern crate origen;

mod dut;
mod file_handler;
mod logger;
mod meta;
mod model;
#[macro_use]
mod pins;
mod registers;
mod services;
#[macro_use]
mod timesets;
mod application;
mod producer;
mod prog_gen;
mod standard_sub_blocks;
mod tester;
mod tester_apis;
mod utility;
mod user;

use crate::registers::bit_collection::BitCollection;
use num_bigint::BigUint;
use origen::{Dut, Error, Operation, Result, Value, FLOW, ORIGEN_CONFIG, STATUS, TEST};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::{PyAny, PyDict, PyBytes};
use pyo3::{wrap_pyfunction, wrap_pymodule};
use std::path::Path;
use std::str::FromStr;
use std::sync::MutexGuard;
use pyo3::conversion::AsPyPointer;

// Imported pyapi modules
use application::PyInit_application;
use dut::PyInit_dut;
use logger::PyInit_logger;
use producer::PyInit_producer;
use prog_gen::interface::PyInit_interface;
use prog_gen::PyInit_prog_gen;
use services::PyInit_services;
use standard_sub_blocks::PyInit_standard_sub_blocks;
use tester::PyInit_tester;
use tester_apis::PyInit_tester_apis;
use utility::location::Location;
use utility::PyInit_utility;
use user::PyInit_users;

#[macro_export]
macro_rules! pypath {
    ($py:expr, $path:expr) => {{
        let locals = [("pathlib", $py.import("pathlib")?)].into_py_dict($py);
        let obj = $py.eval(
            &format!("pathlib.Path(r\"{}\").resolve()", $path),
            None,
            Some(&locals),
        )?;
        obj.to_object($py)
    }};
}

#[pymodule]
/// This is the top-level _origen module which can be imported by Python
fn _origen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(initialize))?;
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(version))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;
    m.add_wrapped(wrap_pyfunction!(clean_mode))?;
    m.add_wrapped(wrap_pyfunction!(target_file))?;
    m.add_wrapped(wrap_pyfunction!(file_handler))?;
    m.add_wrapped(wrap_pyfunction!(test))?;
    m.add_wrapped(wrap_pyfunction!(test_ast))?;
    m.add_wrapped(wrap_pyfunction!(flow))?;
    m.add_wrapped(wrap_pyfunction!(flow_ast))?;
    m.add_wrapped(wrap_pyfunction!(output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_source_directory))?;
    m.add_wrapped(wrap_pyfunction!(on_windows))?;
    m.add_wrapped(wrap_pyfunction!(on_linux))?;
    m.add_wrapped(wrap_pyfunction!(prepare_for_target_load))?;
    m.add_wrapped(wrap_pyfunction!(start_new_test))?;
    m.add_wrapped(wrap_pyfunction!(unhandled_error_count))?;
    m.add_wrapped(wrap_pyfunction!(set_output_dir))?;
    m.add_wrapped(wrap_pyfunction!(set_reference_dir))?;
    m.add_wrapped(wrap_pyfunction!(exit_pass))?;
    m.add_wrapped(wrap_pyfunction!(exit_fail))?;
    m.add_wrapped(wrap_pyfunction!(enable_debug))?;
    m.add_wrapped(wrap_pyfunction!(set_operation))?;

    m.add_wrapped(wrap_pymodule!(logger))?;
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
    m.add_wrapped(wrap_pymodule!(users))?;
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
            trans.address = Some(address.extract::<u128>()?);
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
            // origen::TransactionAction::Capture => trans = value.to_capture_transaction(&dut)?,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::RuntimeError, _>(format!(
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
            trans.address = Some(address.extract::<u128>()?);
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

/// Called automatically when Origen is first loaded
#[pyfunction]
fn initialize(log_verbosity: Option<u8>, cli_location: Option<String>) -> PyResult<()> {
    origen::initialize(log_verbosity, cli_location);
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

/// Returns a file handler object (iterable) for consuming the file arguments
/// given to the CLI
#[pyfunction]
fn file_handler() -> PyResult<file_handler::FileHandler> {
    Ok(file_handler::FileHandler::new())
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
    let _ = ret.set_item("on_windows", cfg!(windows));
    Ok(ret.into())
}

/// Returns the Origen version formatted into PEP440, e.g. "1.2.3.dev4"
#[pyfunction]
fn version() -> PyResult<String> {
    Ok(origen::utility::version::to_pep440(
        &STATUS.origen_version.to_string(),
    )?)
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

/// Returns the Origen application configuration (as defined in application.toml)
#[pyfunction]
fn app_config(py: Python) -> PyResult<PyObject> {
    // let ret = PyDict::new(py);
    // // Don't think an error can really happen here, so not handled
    // let app_config = origen_app_config();
    // let _ = ret.set_item("name", &app_config.name);
    // let _ = ret.set_item("target", &app_config.target);
    // let _ = ret.set_item("mode", &app_config.mode);
    // let _ = ret.set_item("__output_directory__", &app_config.output_directory);
    // let _ = ret.set_item(
    //     "__website_output_directory__",
    //     &app_config.website_output_directory,
    // );
    // let _ = ret.set_item(
    //     "__website_source_directory__",
    //     &app_config.website_source_directory,
    // );
    // let _ = ret.set_item(
    //     "website_release_location",
    //     match &app_config.website_release_location {
    //         Some(loc) => Py::new(py, Location {location: (*loc).clone()}).unwrap().to_object(py),
    //         None => py.None()
    //     }
    // );
    // let _ = ret.set_item(
    //     "website_release_name",
    //     &app_config.website_release_name,
    // );

    let ret = PyDict::new(py);
    let _ = origen::app().unwrap().with_config(|config| {
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
    Ok(ret.into())
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
    let c = origen::core::application::target::clean_name(name, dir, true);
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
fn on_windows() -> PyResult<bool> {
    Ok(origen::core::os::on_windows())
}

#[pyfunction]
fn on_linux() -> PyResult<bool> {
    Ok(origen::core::os::on_linux())
}

#[pyfunction]
/// This will be called by Origen immediately before loading a fresh set of targets
fn prepare_for_target_load() -> PyResult<()> {
    origen::prepare_for_target_load();
    Ok(())
}

#[pyfunction]
/// Clears the current test (pattern) AST and starts a new one
fn start_new_test(name: Option<String>) -> PyResult<()> {
    origen::start_new_test(name);
    Ok(())
}

#[macro_export]
macro_rules! runtime_error {
    ($message:expr) => {{
        Err(PyErr::new::<pyo3::exceptions::RuntimeError, _>(
            $message,
        ))
    }};
}

pub fn pickle(py: Python, object: &impl AsPyPointer) -> PyResult<Vec<u8>> {
    let pickle = PyModule::import(py, "pickle")?;
    pickle.call1("dumps", (object,))?.extract::<Vec<u8>>()
}

pub fn depickle<'a>(py: Python<'a>, object: &Vec<u8>) -> PyResult<&'a PyAny> {
    let pickle = PyModule::import(py, "pickle")?;
    let bytes = PyBytes::new(py, object);
    pickle.call1("loads", (bytes,))
}