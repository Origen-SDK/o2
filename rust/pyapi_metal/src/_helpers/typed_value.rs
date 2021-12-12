use indexmap::IndexMap;
use origen_metal::TypedValue;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use super::pickle::{pickle, depickle};

pub fn typed_value_to_pyobj(data: Option<TypedValue>, key: Option<&str>) -> PyResult<Option<PyObject>> {
    if let Some(d) = data {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match d {
            TypedValue::String(s) => Ok(Some(s.to_object(py))),
            TypedValue::Usize(u) => Ok(Some(u.to_object(py))),
            TypedValue::BigInt(big) => Ok(Some(big.to_object(py))),
            TypedValue::BigUint(big) => Ok(Some(big.to_object(py))),
            TypedValue::Bool(b) => Ok(Some(b.to_object(py))),
            TypedValue::Float(f) => Ok(Some(f.to_object(py))),
            TypedValue::Serialized(bytes, serializer, _class) => {
                if let Some(s) = serializer {
                    if s == "Python-Pickle" {
                        let gil = Python::acquire_gil();
                        let py = gil.python();

                        let any = depickle(py, &bytes)?;
                        Ok(Some(any.into()))
                    } else if s == "Python-Frontend" {
                        let bytes = PyBytes::new(py, &bytes);
                        Ok(Some(bytes.into()))
                    } else {
                        crate::runtime_error!(if let Some(k) = key {
                            format!(
                                "Unknown serializer {} for {}. \
                                    If this was manually serialized, use method 'get_serialized' \
                                    to get a byte-array and manually deserialize.",
                                k, s
                            )
                        } else {
                            format!(
                                "Unknown serializer {}. \
                                    If this was manually serialized, use method 'get_serialized' \
                                    to get a byte-array and manually deserialize.",
                                s
                            )
                        })
                    }
                } else {
                    crate::runtime_error!(if let Some(k) = key {
                        format!(
                            "No serializer provided for {}. \
                                If this was manually serialized, use method 'get_serialized' \
                                to get a byte-array and manually deserialize.",
                            k
                        )
                    } else {
                        "No serializer provided. \
                            If this was manually serialized, use method 'get_serialized' \
                            to get a byte-array and manually deserialize."
                            .to_string()
                    })
                }
            }
            TypedValue::Vec(list) => {
                let mut pylist: Vec<PyObject> = vec![];
                for l in list {
                    pylist.push(typed_value_to_pyobj(Some(l), None)?.unwrap());
                }
                Ok(Some(pylist.to_object(py)))
            }
        }
    } else {
        Ok(None)
    }
}

pub fn extract_as_typed_value(value: &PyAny) -> PyResult<TypedValue> {
    let data;
    if let Ok(s) = value.extract::<String>() {
        data = TypedValue::String(s);
    } else if let Ok(v) = value.extract::<Vec<&PyAny>>() {
        let mut tv_vec: Vec<TypedValue> = vec![];
        for any in v.iter() {
            tv_vec.push(extract_as_typed_value(any)?);
        }
        data = TypedValue::Vec(tv_vec);
    } else if value.get_type().name()?.to_string() == "bool" {
        data = TypedValue::Bool(value.extract::<bool>()?);
    } else if let Ok(bigint) = value.extract::<num_bigint::BigInt>() {
        data = TypedValue::BigInt(bigint);
    } else if let Ok(b) = value.extract::<bool>() {
        data = TypedValue::Bool(b);
    } else if let Ok(f) = value.extract::<f64>() {
        data = TypedValue::Float(f);
    } else {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Serialize the data
        data = TypedValue::Serialized(
            pickle(py, value)?,
            Some("Python-Pickle".to_string()),
            Some(value.get_type().name()?.to_string()),
        );
    }
    Ok(data)
}

// TODO needed?
#[allow(dead_code)]
pub fn from_optional_pydict(
    pydict: Option<&PyDict>,
) -> PyResult<Option<IndexMap<String, TypedValue>>> {
    if let Some(pyd) = pydict {
        Ok(Some(from_pydict(pyd)?))
    } else {
        Ok(None)
    }
}

// TODO needed?
#[allow(dead_code)]
pub fn from_pydict(pydict: &PyDict) -> PyResult<IndexMap<String, TypedValue>> {
    let mut retn = IndexMap::new();
    for (key, val) in pydict.iter() {
        retn.insert(key.extract::<String>()?, extract_as_typed_value(val)?);
    }
    Ok(retn)
}

pub fn into_pydict<'a>(
    py: Python<'a>,
    typed_values: &IndexMap<String, TypedValue>,
) -> PyResult<&'a PyDict> {
    let retn = PyDict::new(py);
    for (key, m) in typed_values {
        retn.set_item(key.clone(), typed_value_to_pyobj(Some(m.clone()), Some(&key))?)?;
    }
    Ok(retn)
}

// TODO needed?
#[allow(dead_code)]
pub fn into_optional_pydict<'a>(
    py: Python<'a>,
    typed_values: Option<&IndexMap<String, TypedValue>>,
) -> PyResult<Option<&'a PyDict>> {
    if let Some(m) = typed_values {
        Ok(Some(into_pydict(py, m)?))
    } else {
        Ok(None)
    }
}

// TODO needed?
#[allow(dead_code)]
pub fn into_optional_pyobj<'a>(
    py: Python<'a>,
    typed_values: Option<&IndexMap<String, TypedValue>>,
) -> PyResult<PyObject> {
    Ok(if let Some(m) = typed_values {
        into_pydict(py, m)?.to_object(py)
    } else {
        py.None()
    })
}
