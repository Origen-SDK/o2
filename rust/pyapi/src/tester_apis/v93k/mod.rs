mod prog_gen_api;

use origen::testers::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct V93K {
    smt_major_version: Option<u32>,
    tester: SupportedTester,
}

#[pymethods]
impl V93K {
    #[new]
    fn new(smt_major_version: Option<u32>) -> PyResult<Self> {
        Ok(V93K {
            smt_major_version: smt_major_version,
            tester: match smt_major_version {
                None => SupportedTester::V93K,
                Some(7) => SupportedTester::V93KSMT7,
                Some(8) => SupportedTester::V93KSMT8,
                Some(ver) => {
                    return Err(PyErr::new::<exceptions::RuntimeError, _>(format!(
                        "SMT version must be 7 or 8, '{}' is not supported",
                        ver
                    )))
                }
            },
        })
    }
}
