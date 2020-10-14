use origen::prog_gen::advantest::common as v93k_prog;
use origen::testers::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct V93K {
    smt_major_version: u32,
    tester_type: SupportedTester,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    id: usize,
    name: String,
    tester: SupportedTester,
}

#[pymethods]
impl V93K {
    #[new]
    fn new(obj: &PyRawObject, smt_major_version: u32) -> PyResult<()> {
        obj.init({
            V93K {
                smt_major_version: smt_major_version,
                tester_type: match smt_major_version {
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

    pub fn new_test_suite(&self, name: &str) -> PyResult<Test> {
        Ok(Test {
            name: name.to_string(),
            id: v93k_prog::new_test_suite(name, &self.tester_type)?,
            tester: self.tester_type.clone(),
        })
    }
}
