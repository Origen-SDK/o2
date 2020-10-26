mod test_methods;
mod test_suites;

use origen::testers::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

use test_methods::TestMethods;
pub use test_suites::TestSuites;
pub use test_suites::TestSuite;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct V93K {
    smt_major_version: u32,
    tester: SupportedTester,
}

#[pymethods]
impl V93K {
    #[new]
    fn new(obj: &PyRawObject, smt_major_version: u32) -> PyResult<()> {
        obj.init({
            V93K {
                smt_major_version: smt_major_version,
                tester: match smt_major_version {
                    7 => SupportedTester::V93KSMT7,
                    8 => SupportedTester::V93KSMT8,
                    _ => {
                        return Err(PyErr::new::<exceptions::RuntimeError, _>(format!(
                            "SMT version must be 7 or 8, '{}' is not supported",
                            smt_major_version
                        )))
                    }
                },
            }
        });
        Ok(())
    }

    #[getter]
    pub fn test_suites(&self) -> PyResult<TestSuites> {
        Ok(TestSuites {
            tester: self.tester.clone(),
        })
    }

    #[getter]
    pub fn test_methods(&self) -> PyResult<TestMethods> {
        Ok(TestMethods {
            tester: self.tester.clone(),
        })
    }
}
