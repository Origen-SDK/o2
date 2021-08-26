use super::to_param_value;
use super::Test;
use crate::prog_gen::flow_options;
use crate::utility::caller::src_caller_meta;
use origen::prog_gen::{flow_api, Limit, LimitSelector, ParamValue};
use origen::testers::SupportedTester;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions::PyAttributeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// A test invocation models a particular call (or invocation) of a test from a test flow,
/// on the V93K a TestInvocation maps to a TestSuite, while a Test maps to a TestMethod.
/// See the Test description for comments on the multiple ID fields.
#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInvocation {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

#[pymethods]
impl TestInvocation {
    pub fn set_test_obj(&mut self, test: Test) -> PyResult<()> {
        if !test.tester.is_compatible_with(&self.tester) {
            return Err(PyAttributeError::new_err(format!(
                "Attempted to associate a test for '{}' with an invocation for '{}'",
                test.tester, self.tester
            )));
        }
        flow_api::assign_test_to_invocation(self.id, test.id, src_caller_meta())?;
        Ok(())
    }

    #[setter]
    pub fn set_lo_limit(&self, value: &PyAny) -> PyResult<()> {
        let value = match to_param_value(value)? {
            None => None,
            Some(x) => Some(Limit {
                kind: origen::prog_gen::LimitType::GTE,
                value: x,
                unit: None,
            }),
        };
        flow_api::set_test_limit(
            None,
            Some(self.id),
            LimitSelector::Lo,
            value,
            src_caller_meta(),
        )?;
        Ok(())
    }

    #[setter]
    pub fn set_hi_limit(&self, value: &PyAny) -> PyResult<()> {
        let value = match to_param_value(value)? {
            None => None,
            Some(x) => Some(Limit {
                kind: origen::prog_gen::LimitType::LTE,
                value: x,
                unit: None,
            }),
        };
        flow_api::set_test_limit(
            None,
            Some(self.id),
            LimitSelector::Hi,
            value,
            src_caller_meta(),
        )?;
        Ok(())
    }
}

impl TestInvocation {
    pub fn new(
        name: String,
        tester: SupportedTester,
        kwargs: Option<&PyDict>,
    ) -> Result<TestInvocation> {
        let id = flow_api::define_test_invocation(&name, &tester, src_caller_meta())?;

        let t = TestInvocation {
            name: name,
            tester: tester,
            id: id,
        };

        if let Some(kwargs) = kwargs {
            for (k, v) in kwargs {
                if let Ok(name) = k.extract::<String>() {
                    if !flow_options::is_flow_option(&name) {
                        if name == "lo_limit" {
                            t.set_lo_limit(v)?;
                        } else if name == "hi_limit" {
                            t.set_hi_limit(v)?;
                        } else {
                            t.set_attr(&name, to_param_value(v)?)?;
                        }
                    }
                } else {
                    return error!("Illegal attribute name type '{}', should be a String", k);
                }
            }
        }
        Ok(t)
    }

    pub fn set_attr(&self, name: &str, value: Option<ParamValue>) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, src_caller_meta())?;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for TestInvocation {
    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        // Specials for platform specific attributes
        if name == "test_method"
            && (self.tester == SupportedTester::V93KSMT7
                || self.tester == SupportedTester::V93KSMT8
                || self.tester == SupportedTester::V93K)
        {
            let test = value.extract::<Test>()?;
            return self.set_test_obj(test);
        }
        self.set_attr(name, to_param_value(value)?)?;
        Ok(())
    }
}
