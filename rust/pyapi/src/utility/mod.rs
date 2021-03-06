pub mod caller;
pub mod ldap;
pub mod linter;
pub mod location;
#[allow(non_snake_case)]
pub mod mailer;
pub mod metadata;
pub mod publisher;
pub mod results;
pub mod revision_control;
pub mod session_store;
pub mod transaction;
pub mod unit_testers;
pub mod version;
pub mod website;

use ldap::PyInit_ldap;
use linter::PyInit_linter;
use location::Location;
use mailer::PyInit_mailer;
use publisher::PyInit_publisher;
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use results::PyInit_results;
use revision_control::PyInit_revision_control;
use session_store::PyInit_session_store;
use transaction::Transaction;
use unit_testers::PyInit_unit_testers;
use version::Version;
use website::PyInit_website;

use crate::_helpers::hashmap_to_pydict;
use crate::runtime_error;
use num_bigint::BigUint;
use origen::utility::big_uint_helpers::BigUintHelpers;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::path::PathBuf;

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Location>()?;
    m.add_class::<Transaction>()?;
    m.add_class::<Version>()?;
    m.add_wrapped(wrap_pyfunction!(reverse_bits))?;
    m.add_wrapped(wrap_pymodule!(mailer))?;
    m.add_wrapped(wrap_pymodule!(session_store))?;
    m.add_wrapped(wrap_pymodule!(ldap))?;
    m.add_wrapped(wrap_pymodule!(revision_control))?;
    m.add_wrapped(wrap_pymodule!(unit_testers))?;
    m.add_wrapped(wrap_pymodule!(publisher))?;
    m.add_wrapped(wrap_pymodule!(linter))?;
    m.add_wrapped(wrap_pymodule!(results))?;
    m.add_wrapped(wrap_pymodule!(website))?;
    m.add_wrapped(wrap_pyfunction!(exec))?;
    m.add_wrapped(wrap_pyfunction!(dispatch_workflow))?;
    Ok(())
}

#[pyfunction]
pub fn reverse_bits(_py: Python, num: BigUint, width: Option<u64>) -> PyResult<BigUint> {
    Ok(num.reverse(width.unwrap_or(num.bits()) as usize)?)
}

#[pyfunction(
    capture = "true",
    timeout = "None",
    cd = "None",
    add_env = "None",
    remove_env = "None",
    clear_env = "false"
)]
pub fn exec(
    _py: Python,
    cmd: Vec<String>,
    capture: bool,
    timeout: Option<u32>,
    cd: Option<String>,
    add_env: Option<HashMap<String, String>>,
    remove_env: Option<Vec<String>>,
    clear_env: bool,
) -> PyResult<results::ExecResult> {
    let result = origen::utility::command_helpers::exec(
        cmd,
        capture,
        {
            if let Some(t) = timeout {
                Some(std::time::Duration::new(t as u64, 0))
            } else {
                None
            }
        },
        {
            if let Some(d) = cd {
                Some(PathBuf::from(d))
            } else {
                None
            }
        },
        add_env,
        remove_env,
        clear_env,
    )?;
    Ok(results::ExecResult {
        exec_result: Some(result),
    })
}

fn new_obj(py: Python, class: &str, kwargs: &PyDict) -> PyResult<PyObject> {
    let split = class.rsplitn(2, ".").collect::<Vec<&str>>();
    let locals = PyDict::new(py);
    locals.set_item("kwargs", kwargs)?;
    let mut class_mod = "";
    if let Some(m) = split.get(1) {
        locals.set_item("mod", py.import(m)?.to_object(py))?;
        class_mod = "mod."
    }

    let obj = py.eval(
        &format!("{}{}(**kwargs)", class_mod, split[0]),
        Some(locals),
        None,
    )?;
    Ok(obj.to_object(py))
}

fn app_utility<F>(
    name: &str,
    config: Option<&HashMap<String, String>>,
    callback_function: Option<F>,
) -> PyResult<Option<PyObject>>
where
    F: FnMut(Option<&HashMap<String, String>>) -> PyResult<Option<PyObject>>,
{
    let gil = Python::acquire_gil();
    let py = gil.python();
    if let Some(conf) = config.as_ref() {
        if let Some(c) = conf.get("system") {
            // Get the module and try to import it
            let split = c.rsplitn(2, ".");
            if split.count() == 2 {
                // Have a class (hopefully) of the form 'a.b.Class'
                let py_conf = hashmap_to_pydict(py, conf)?;
                Ok(Some(new_obj(py, c, py_conf)?))
            } else {
                // fall back to some enumerated systems
                if &c.to_lowercase() == "none" {
                    // "none" always implies no system
                    Ok(None)
                } else {
                    if let Some(mut cb) = callback_function {
                        cb(config)
                    } else {
                        return runtime_error!(format!("Unrecognized {} system '{}'", name, c));
                    }
                }
            }
        } else {
            // Invalid config
            return runtime_error!(format!("Could not discern {} from app config", name));
        }
    } else {
        if let Some(mut cb) = callback_function {
            Ok(cb(config)?)
        } else {
            Ok(None)
        }
    }
}

#[pyfunction(inputs = "None")]
pub fn dispatch_workflow(
    owner: &str,
    repo: &str,
    workflow: &str,
    git_ref: &str,
    inputs: Option<HashMap<String, String>>,
) -> PyResult<results::GenericResult> {
    let res = origen::utility::github::dispatch_workflow(owner, repo, workflow, git_ref, inputs)?;
    Ok(results::GenericResult::from_origen(res))
}
