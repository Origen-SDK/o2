use pyo3::prelude::*;

#[pymodule]
/// Implements the module _origen.services in Python
pub fn services(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<JTAG>()?;

    Ok(())
}

#[pyclass]
struct JTAG {
    id: usize,
}

#[pymethods]
impl JTAG {
    #[new]
    fn new(_obj: &PyRawObject) -> Self {
        JTAG { id: 0 }
    }
}
