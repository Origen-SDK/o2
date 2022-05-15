pub mod caller;
pub mod ldaps;
pub mod linter;
pub mod location;
#[allow(non_snake_case)]
pub mod mailer;
pub mod publisher;
pub mod release_scribe;
pub mod results;
pub mod revision_control;
pub mod sessions;
pub mod transaction;
pub mod unit_testers;
pub mod version;
pub mod website;

use ldaps::__pyo3_get_function_ldaps;
use linter::PyInit_linter;
use location::Location;
use mailer::PyInit_mailer;
use publisher::PyInit_publisher;
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use release_scribe::PyInit_release_scribe;
use results::PyInit_results;
use revision_control::PyInit_revision_control;
use sessions::PyInit_sessions;
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

// TODO move this to somewhere better
#[macro_export]
macro_rules! optional_config_value_map_into_pydict {
    ($py:expr, $map:expr) => {
        if let Some(m) = $map {
            crate::utility::config_value_map_into_pydict($py, &mut m.iter())?.to_object($py)
        } else {
            $py.None()
        }
    };
}

#[pymodule]
pub fn utility(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Location>()?;
    m.add_class::<Transaction>()?;
    m.add_class::<Version>()?;
    m.add_wrapped(wrap_pyfunction!(reverse_bits))?;
    m.add_wrapped(wrap_pymodule!(mailer))?;
    m.add_wrapped(wrap_pymodule!(sessions))?;
    m.add_wrapped(wrap_pyfunction!(ldaps))?;
    m.add_wrapped(wrap_pymodule!(revision_control))?;
    m.add_wrapped(wrap_pymodule!(unit_testers))?;
    m.add_wrapped(wrap_pymodule!(publisher))?;
    m.add_wrapped(wrap_pymodule!(linter))?;
    m.add_wrapped(wrap_pymodule!(release_scribe))?;
    m.add_wrapped(wrap_pymodule!(results))?;
    m.add_wrapped(wrap_pymodule!(website))?;
    m.add_wrapped(wrap_pyfunction!(exec))?;
    m.add_wrapped(wrap_pyfunction!(dispatch_workflow))?;
    Ok(())
}

// TODO needed? or move to OM?
// pub fn to_pylist<'p, I>(py: Python<'p>, list: &'p mut dyn Iterator<Item=I>) -> PyResult<Py<pyo3::types::PyList>>
// where
//     I: 'p + ToPyObject
// {
//     let pylist = pyo3::types::PyList::empty(py);
//     for i in list {
//         pylist.append(i)?;
//     }
//     Ok(pylist.into())
// }

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

// TODO use metal's
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

fn app_utility(
    name: &str,
    config: Option<&HashMap<String, String>>,
    default: Option<&str>,
    use_by_default: bool,
) -> PyResult<Option<PyObject>> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let system: &str;
    let conf_t: HashMap<String, String>;
    let conf_;
    if let Some(conf) = config {
        if let Some(c) = conf.get("system") {
            system = c;
        } else {
            if let Some(s) = default {
                system = s;
            } else {
                return runtime_error!(format!(
                    "Could not discern {} from the app config! \
                    No 'system' was specified and no default was given!",
                    name
                ));
            }
        }
        conf_ = conf;
    } else {
        if use_by_default {
            if let Some(s) = default {
                system = s;
                conf_t = HashMap::new();
                conf_ = &conf_t;
            } else {
                return runtime_error!(format!(
                    "Could not discern {} from the app config! \
                     Expected a default system but none was given!",
                    name
                ));
            }
        } else {
            return Ok(None);
        }
    }

    // Get the module and try to import it
    let split = system.rsplitn(2, ".");
    if split.count() == 2 {
        // Have a class (hopefully) of the form 'a.b.Class'
        let py_conf = hashmap_to_pydict(py, conf_)?;
        Ok(Some(new_obj(py, system, py_conf)?))
    } else {
        // fall back to some enumerated systems
        if &system.to_lowercase() == "none" {
            // "none" always implies no system
            Ok(None)
        } else {
            runtime_error!(format!("Unrecognized {} system '{}'", name, system))
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

// TODO relocate these
use origen_metal::config;
use pyo3::types::PyList;

pub fn config_value_into_pyobject(py: Python, v: &config::Value) -> PyResult<PyObject> {
    Ok(match &v.kind {
        config::ValueKind::Boolean(b) => b.to_object(py),
        config::ValueKind::I64(i) => i.to_object(py),
        config::ValueKind::I128(i) => i.to_object(py),
        config::ValueKind::U64(u) => u.to_object(py),
        config::ValueKind::U128(u) => u.to_object(py),
        config::ValueKind::Float(f) => f.to_object(py),
        config::ValueKind::String(s) => s.to_object(py),
        config::ValueKind::Table(map) => {
            let pydict = PyDict::new(py);
            for (k, inner_v) in map.iter() {
                pydict.set_item(k, config_value_into_pyobject(py, inner_v)?)?;
            }
            pydict.to_object(py)
        }
        config::ValueKind::Array(vec) => {
            let pylist = PyList::empty(py);
            for inner_v in vec.iter() {
                pylist.append(config_value_into_pyobject(py, inner_v)?)?;
            }
            pylist.to_object(py)
        }
        config::ValueKind::Nil => {
            return runtime_error!(format!(
                "Cannot convert config value '{}' to a Python object",
                v
            ))
        }
    })
}

pub fn config_value_map_into_pydict<'p>(
    py: Python<'p>,
    map: &'p mut dyn Iterator<Item = (&String, &config::Value)>,
) -> PyResult<Py<PyDict>> {
    let pydict = PyDict::new(py);
    for (k, v) in map.into_iter() {
        pydict.set_item(
            k,
            match config_value_into_pyobject(py, v) {
                Ok(o) => o,
                Err(_e) => {
                    return runtime_error!(format!(
                        "Cannot convert config value '{}' with value '{}' to a Python object",
                        k, v
                    ))
                }
            },
        )?;
    }
    Ok(pydict.into())
}
