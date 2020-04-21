mod dut;
mod file_handler;
mod logger;
mod meta;
mod model;
mod pins;
mod registers;
mod services;
#[macro_use]
mod timesets;
mod producer;
mod tester;

use crate::registers::bit_collection::BitCollection;
use num_bigint::BigUint;
use origen::app_config as origen_app_config;
use origen::{Dut, Error, Result, Value, ORIGEN_CONFIG, STATUS, TEST};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::{PyAny, PyDict};
use pyo3::{wrap_pyfunction, wrap_pymodule};
use std::sync::MutexGuard;

// Imported pyapi modules
use dut::PyInit_dut;
use logger::PyInit_logger;
use producer::PyInit_producer;
use services::PyInit_services;
use tester::PyInit_tester;

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
    m.add_wrapped(wrap_pyfunction!(status))?;
    m.add_wrapped(wrap_pyfunction!(config))?;
    m.add_wrapped(wrap_pyfunction!(app_config))?;
    m.add_wrapped(wrap_pyfunction!(clean_mode))?;
    m.add_wrapped(wrap_pyfunction!(target_file))?;
    m.add_wrapped(wrap_pyfunction!(file_handler))?;
    m.add_wrapped(wrap_pyfunction!(test))?;
    m.add_wrapped(wrap_pyfunction!(test_ast))?;
    m.add_wrapped(wrap_pyfunction!(output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_output_directory))?;
    m.add_wrapped(wrap_pyfunction!(website_source_directory))?;
    m.add_wrapped(wrap_pyfunction!(on_windows))?;
    m.add_wrapped(wrap_pyfunction!(on_linux))?;

    m.add_wrapped(wrap_pymodule!(logger))?;
    m.add_wrapped(wrap_pymodule!(dut))?;
    m.add_wrapped(wrap_pymodule!(tester))?;
    m.add_wrapped(wrap_pymodule!(producer))?;
    m.add_wrapped(wrap_pymodule!(services))?;
    Ok(())
}

fn extract_value<'a>(
    bits_or_val: &PyAny,
    size: Option<u32>,
    dut: &'a MutexGuard<Dut>,
) -> Result<Value<'a>> {
    let bits = bits_or_val.extract::<&BitCollection>();
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
    let _ = ret.set_item("root", format!("{}", STATUS.root.display()));
    let _ = ret.set_item("origen_version", &STATUS.origen_version.to_string());
    let _ = ret.set_item("home", format!("{}", STATUS.home.display()));
    let _ = ret.set_item("on_windows", cfg!(windows));
    Ok(ret.into())
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
    let ret = PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let app_config = origen_app_config();
    let _ = ret.set_item("name", &app_config.name);
    let _ = ret.set_item("target", &app_config.target);
    let _ = ret.set_item("mode", &app_config.mode);
    let _ = ret.set_item("__output_directory__", &app_config.output_directory);
    let _ = ret.set_item(
        "__website_output_directory__",
        &app_config.website_output_directory,
    );
    let _ = ret.set_item(
        "__website_source_directory__",
        &app_config.website_source_directory,
    );
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
    let dir = origen::core::application::output_directory();
    Ok(pypath!(py, dir.display()))
}

#[pyfunction]
fn website_output_directory(py: Python) -> PyResult<PyObject> {
    let dir = origen::core::application::website_output_directory();
    Ok(pypath!(py, dir.display()))
}

#[pyfunction]
fn website_source_directory(py: Python) -> PyResult<PyObject> {
    let dir = origen::core::application::website_source_directory();
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
