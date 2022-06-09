/// Helpers to transition from the config values to PyObjects

use pyo3::prelude::*;
use origen_metal::config;
use pyo3::types::{PyDict, PyList};
use crate::runtime_error;

#[macro_export]
macro_rules! optional_config_value_map_into_pydict {
    ($py:expr, $map:expr) => {
        if let Some(m) = $map {
            pyapi_metal::_helpers::config::config_value_map_into_pydict($py, &mut m.iter())?.to_object($py)
        } else {
            $py.None()
        }
    };
}

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
