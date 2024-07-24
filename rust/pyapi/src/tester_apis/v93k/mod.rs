mod prog_gen_api;

use origen_metal::prog_gen::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

#[pyclass(subclass)]
/// Python interface for the tester backend.
pub struct V93K {
    tester: SupportedTester,
}

#[pymethods]
impl V93K {
    #[new]
    fn new(smt_major_version: Option<u32>) -> PyResult<Self> {
        Ok(V93K {
            tester: match smt_major_version {
                None => SupportedTester::V93K,
                Some(7) => SupportedTester::V93KSMT7,
                Some(8) => SupportedTester::V93KSMT8,
                Some(ver) => {
                    return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                        "SMT version must be 7 or 8, '{}' is not supported",
                        ver
                    )))
                }
            },
        })
    }
}
