//! Implements Python bindings for program generation data structures and functions

pub mod interface;

use origen::prog_gen::Test as RichTest;
use origen::prog_gen::{ParamType, ParamValue};
use origen::testers::SupportedTester;
use origen::{Result, PROG};
use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions::AttributeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub tester: SupportedTester,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInvocation {
    pub id: usize,
    pub test_id: Option<usize>,
    pub tester: SupportedTester,
}

#[pymethods]
impl TestInvocation {
    fn set_test_obj(&mut self, test: Test) -> PyResult<()> {
        if test.tester != self.tester {
            //return error!("Blah");
        }
        log_info!("Set test object called!");
        self.test_id = Some(test.id);
        Ok(())
    }

    fn name(&self) -> PyResult<String> {
        Ok(PROG
            .for_tester(&self.tester)
            .with_test(self.id, |t| Ok(t.name.to_owned()))?)
    }

    fn test_name(&self) -> PyResult<Option<String>> {
        if let Some(id) = self.test_id {
            Ok(PROG
                .for_tester(&self.tester)
                .with_test(id, |t| Ok(Some(t.name.to_owned())))?)
        } else {
            Ok(None)
        }
    }
}

#[pyproto]
impl PyObjectProtocol for TestInvocation {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<()> {
        dbg!(query);
        Ok(())
    }

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        if name == "test_method"
            && (self.tester == SupportedTester::V93KSMT7
                || self.tester == SupportedTester::V93KSMT8)
        {
            self.set_test_obj(value.extract::<Test>()?)?;
            return Ok(());
        }
        if PROG.for_tester(&self.tester).with_test_mut(self.id, |t| {
            if t.has_param(name) {
                set_value(t, name, value)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })? {
            return Ok(());
        }
        if let Some(id) = self.test_id {
            if PROG.for_tester(&self.tester).with_test_mut(id, |t| {
                if t.has_param(name) {
                    set_value(t, name, value)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            })? {
                return Ok(());
            }
        }
        let msg = match self.test_id {
            Some(id) => format!(
                "Neither the {} '{}' or its {} '{}' has an attribute called '{}'",
                name_of_test_invocation(&self.tester),
                &self.name()?,
                name_of_test(&self.tester),
                &self.test_name()?.unwrap(),
                name
            ),
            None => format!(
                "The {} '{}' has no attribute called '{}'",
                name_of_test_invocation(&self.tester),
                &self.name()?,
                name
            ),
        };
        Err(AttributeError::py_err(msg))
    }
}

fn name_of_test(tester: &SupportedTester) -> &str {
    match tester {
        SupportedTester::V93KSMT7 | SupportedTester::V93KSMT8 => "Test Method",
        _ => "Test",
    }
}

fn name_of_test_invocation(tester: &SupportedTester) -> &str {
    match tester {
        SupportedTester::V93KSMT7 | SupportedTester::V93KSMT8 => "Test Suite",
        _ => "Test Invocation",
    }
}

fn set_value(t: &mut RichTest, name: &str, value: &PyAny) -> Result<()> {
    let _ = match t.get_type(name)? {
        ParamType::Bool => {
            if let Ok(v) = value.extract::<bool>() {
                t.set(name, ParamValue::Bool(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a Boolean, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Int => {
            if let Ok(v) = value.extract::<i64>() {
                t.set(name, ParamValue::Int(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected an Integer, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::UInt => {
            if let Ok(v) = value.extract::<u64>() {
                t.set(name, ParamValue::UInt(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected an Unsigned Integer, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Float => {
            if let Ok(v) = value.extract::<f64>() {
                t.set(name, ParamValue::Float(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a FLoat, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Current => {
            if let Ok(v) = value.extract::<f64>() {
                t.set(name, ParamValue::Current(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a FLoat, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Voltage => {
            if let Ok(v) = value.extract::<f64>() {
                t.set(name, ParamValue::Voltage(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a FLoat, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Time => {
            if let Ok(v) = value.extract::<f64>() {
                t.set(name, ParamValue::Time(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a FLoat, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::Frequency => {
            if let Ok(v) = value.extract::<f64>() {
                t.set(name, ParamValue::Frequency(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a FLoat, got: '{}'", name, &t.name, value);
            }
        }
        ParamType::String => {
            if let Ok(v) = value.extract::<String>() {
                t.set(name, ParamValue::String(v))?
            } else {
                return error!("Illegal value applied to attribute '{}' of test '{}', expected a String, got: '{}'", name, &t.name, value);
            }
        }
    };
    Ok(())
}
