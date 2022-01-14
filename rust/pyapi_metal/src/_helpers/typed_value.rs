use indexmap::IndexMap;
use origen_metal::{TypedValue, TypedValueMap, TypedValueVec};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyTuple, PyList};
use super::pickle::{pickle, depickle};

pub use typed_value_to_pyobj as to_pyobject;
pub use extract_as_typed_value as from_pyany;

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

pub fn from_pylist(pylist: &PyList) -> PyResult<TypedValueVec> {
    pylist.iter().map(|i| extract_as_typed_value(i)).collect::<PyResult<TypedValueVec>>()
}

pub fn from_optional_pylist(
    pylist: Option<&PyList>,
) -> PyResult<Option<TypedValueVec>> {
    if let Some(pyl) = pylist {
        Ok(Some(from_pylist(pyl)?))
    } else {
        Ok(None)
    }
}

pub fn into_pytuple<'a>(
    py: Python<'a>,
    typed_values: &mut dyn Iterator<Item=&TypedValue>
) -> PyResult<&'a PyTuple> {
    Ok(PyTuple::new(py, typed_values.map(|tv| typed_value_to_pyobj(Some(tv.clone()), None)).collect::<PyResult<Vec<Option<PyObject>>>>()?))
}

#[allow(dead_code)]
pub fn into_optional_pytuple<'a>(py: Python<'a>, typed_values: Option<&mut dyn Iterator<Item=&TypedValue>>) -> PyResult<Option<&'a PyTuple>> {
    Ok(if let Some(tv) = typed_values {
        Some(into_pytuple(py, tv)?)
    } else {
        None
    })
}

#[allow(dead_code)]
pub fn into_pylist<'a>(
    py: Python<'a>,
    typed_values: &Vec<TypedValue>,
) -> PyResult<&'a PyList> {
    Ok(PyList::new(py, typed_values.iter().map(|tv| typed_value_to_pyobj(Some(tv.clone()), None)).collect::<PyResult<Vec<Option<PyObject>>>>()))
}

pub fn from_optional_pydict(
    pydict: Option<&PyDict>,
) -> PyResult<Option<TypedValueMap>> {
    if let Some(pyd) = pydict {
        Ok(Some(from_pydict(pyd)?))
    } else {
        Ok(None)
    }
}

pub fn from_pydict(pydict: &PyDict) -> PyResult<TypedValueMap> {
    let mut retn = TypedValueMap::new();
    for (key, val) in pydict.iter() {
        retn.insert(key.extract::<&str>()?, extract_as_typed_value(val)?);
    }
    Ok(retn)
}

pub fn into_pydict<'a>(
    py: Python<'a>,
    typed_values: impl Into<TypedValueMap>
) -> PyResult<&'a PyDict> {
    let t = typed_values.into();
    let retn = PyDict::new(py);
    for (key, m) in t.typed_values() {
        retn.set_item(key.clone(), typed_value_to_pyobj(Some(m.clone()), Some(&key))?)?;
    }
    Ok(retn)
}

pub fn into_optional_pydict<'a>(
    py: Python<'a>,
    typed_values: Option<impl Into<TypedValueMap>>
) -> PyResult<Option<&'a PyDict>> {
    if let Some(m) = typed_values {
        Ok(Some(into_pydict(py, m)?))
    } else {
        Ok(None)
    }
}

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
