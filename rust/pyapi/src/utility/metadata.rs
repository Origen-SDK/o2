use indexmap::IndexMap;
use origen::Metadata;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

pub fn metadata_to_pyobj(data: Option<Metadata>, key: Option<&str>) -> PyResult<Option<PyObject>> {
    if let Some(d) = data {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match d {
            Metadata::String(s) => Ok(Some(s.to_object(py))),
            Metadata::Usize(u) => Ok(Some(u.to_object(py))),
            Metadata::BigInt(big) => Ok(Some(big.to_object(py))),
            Metadata::BigUint(big) => Ok(Some(big.to_object(py))),
            Metadata::Bool(b) => Ok(Some(b.to_object(py))),
            Metadata::Float(f) => Ok(Some(f.to_object(py))),
            Metadata::Serialized(bytes, serializer, _class) => {
                if let Some(s) = serializer {
                    if s == "Python-Pickle" {
                        let gil = Python::acquire_gil();
                        let py = gil.python();

                        let any = crate::depickle(py, &bytes)?;
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
            Metadata::Vec(list) => {
                let mut pylist: Vec<PyObject> = vec![];
                for l in list {
                    pylist.push(metadata_to_pyobj(Some(l), None)?.unwrap());
                }
                Ok(Some(pylist.to_object(py)))
            }
        }
    } else {
        Ok(None)
    }
}

pub fn extract_as_metadata(value: &PyAny) -> PyResult<Metadata> {
    let data;
    if let Ok(s) = value.extract::<String>() {
        data = Metadata::String(s);
    } else if let Ok(v) = value.extract::<Vec<&PyAny>>() {
        let mut metadata_vec: Vec<Metadata> = vec![];
        for any in v.iter() {
            metadata_vec.push(extract_as_metadata(any)?);
        }
        data = Metadata::Vec(metadata_vec);
    } else if value.get_type().name().to_string() == "bool" {
        data = Metadata::Bool(value.extract::<bool>()?);
    } else if let Ok(bigint) = value.extract::<num_bigint::BigInt>() {
        data = Metadata::BigInt(bigint);
    } else if let Ok(b) = value.extract::<bool>() {
        data = Metadata::Bool(b);
    } else if let Ok(f) = value.extract::<f64>() {
        data = Metadata::Float(f);
    } else {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Serialize the data
        data = Metadata::Serialized(
            crate::pickle(py, value)?,
            Some("Python-Pickle".to_string()),
            Some(value.get_type().name().to_string()),
        );
    }
    Ok(data)
}

pub fn from_optional_pydict(
    pydict: Option<&PyDict>,
) -> PyResult<Option<IndexMap<String, Metadata>>> {
    if let Some(pyd) = pydict {
        Ok(Some(from_pydict(pyd)?))
    } else {
        Ok(None)
    }
}

pub fn from_pydict(pydict: &PyDict) -> PyResult<IndexMap<String, Metadata>> {
    let mut retn = IndexMap::new();
    for (key, val) in pydict.iter() {
        retn.insert(key.extract::<String>()?, extract_as_metadata(val)?);
    }
    Ok(retn)
}

pub fn into_pydict<'a>(
    py: Python<'a>,
    metadata: &IndexMap<String, Metadata>,
) -> PyResult<&'a PyDict> {
    let retn = PyDict::new(py);
    for (key, m) in metadata {
        retn.set_item(key.clone(), metadata_to_pyobj(Some(m.clone()), Some(&key))?)?;
    }
    Ok(retn)
}

#[allow(dead_code)]
pub fn into_optional_pydict<'a>(
    py: Python<'a>,
    metadata: Option<&IndexMap<String, Metadata>>,
) -> PyResult<Option<&'a PyDict>> {
    if let Some(m) = metadata {
        Ok(Some(into_pydict(py, m)?))
    } else {
        Ok(None)
    }
}

pub fn into_optional_pyobj<'a>(
    py: Python<'a>,
    metadata: Option<&IndexMap<String, Metadata>>,
) -> PyResult<PyObject> {
    Ok(if let Some(m) = metadata {
        into_pydict(py, m)?.to_object(py)
    } else {
        py.None()
    })
}
