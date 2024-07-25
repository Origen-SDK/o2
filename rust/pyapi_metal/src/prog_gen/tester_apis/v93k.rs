use crate::prog_gen::{Test, TestInvocation};
use origen_metal::prog_gen::{ParamValue, SupportedTester};
use pyo3::{exceptions, prelude::*};
use pyo3::types::PyDict;


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

    #[pyo3(signature=(name, library, **kwargs))]
    fn new_test_method(
        &mut self,
        name: String,
        library: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        let t = Test::new(name.clone(), self.tester.to_owned(), library, name, kwargs)?;
        Ok(t)
    }

    #[pyo3(signature=(name, **kwargs))]
    fn new_test_suite(
        &mut self,
        name: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<TestInvocation> {
        let t = TestInvocation::new(name.clone(), self.tester.to_owned(), kwargs)?;
        t.set_attr("name", Some(ParamValue::String(name.to_owned())))?;
        Ok(t)
    }
}
