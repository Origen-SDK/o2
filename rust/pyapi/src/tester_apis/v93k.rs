use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct V93K {
    smt_major_version: u32,
}

#[pymethods]
impl V93K {
    #[new]
    fn new(_obj: &PyRawObject, smt_major_version: u32) -> Self {
        V93K {
            smt_major_version: smt_major_version,
        }
    }

    //fn new_test_suite()
}
