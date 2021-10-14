use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

pub fn hashmap_to_pydict<'p>(
    py: Python<'p>,
    hmap: &HashMap<String, String>,
) -> PyResult<&'p PyDict> {
    let py_config = PyDict::new(py);
    for (k, v) in hmap.iter() {
        py_config.set_item(k, v)?;
    }
    Ok(py_config)
}
