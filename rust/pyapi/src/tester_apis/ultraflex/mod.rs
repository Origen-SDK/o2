mod test_instances;

use origen::testers::SupportedTester;
//use pyo3::exceptions;
use pyo3::prelude::*;

use test_instances::TestInstances;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct ULTRAFLEX {
    tester: SupportedTester,
}

#[pymethods]
impl ULTRAFLEX {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(ULTRAFLEX {
            tester: SupportedTester::ULTRAFLEX,
        })
    }

    #[getter]
    pub fn test_instances(&self) -> PyResult<TestInstances> {
        Ok(TestInstances {
            tester: self.tester.clone(),
        })
    }
}
