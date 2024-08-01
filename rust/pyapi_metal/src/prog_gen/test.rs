use super::to_param_value;
use crate::prog_gen::flow_options;
use super::src_caller_meta;
use origen_metal::prog_gen::{flow_api, Limit, LimitSelector, ParamValue, SupportedTester};
use origen_metal::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

impl Test {
    pub fn new(
        name: String,
        tester: SupportedTester,
        library_name: String,
        template_name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> Result<Test> {
        let id = flow_api::define_test(
            &name,
            &tester,
            &library_name,
            &template_name,
            src_caller_meta(),
        )?;

        let t = Test {
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
                            t._set_attr(&name, to_param_value(v)?, allow_missing)?;
                        }
                    }
                } else {
                    bail!("Illegal attribute name type '{}', should be a String", k);
                }
            }
        }

        Ok(t)
    }

    pub fn _set_attr(&self, name: &str, value: Option<ParamValue>, allow_missing: bool) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, allow_missing, src_caller_meta())?;
        Ok(())
    }
}

#[pymethods]
impl Test {
    #[setter]
    pub fn set_lo_limit(&self, value: &PyAny) -> PyResult<()> {
        let value = match to_param_value(value)? {
            None => None,
            Some(x) => Some(Limit {
                kind: origen_metal::prog_gen::LimitType::GTE,
                value: x,
                unit: None,
            }),
        };
        flow_api::set_test_limit(
            Some(self.id),
            None,
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
                kind: origen_metal::prog_gen::LimitType::LTE,
                value: x,
                unit: None,
            }),
        };
        flow_api::set_test_limit(
            Some(self.id),
            None,
            LimitSelector::Hi,
            value,
            src_caller_meta(),
        )?;
        Ok(())
    }

    #[pyo3(signature=(name, value, allow_missing=false))]
    pub fn set_attr(&self, name: &str, value: Option<&PyAny>, allow_missing: bool) -> Result<()> {
        let value = match value {
            Some(x) => to_param_value(x)?,
            None => None,
        };
        flow_api::set_test_attr(self.id, name, value, allow_missing, src_caller_meta())?;
        Ok(())
    }

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        self._set_attr(name, to_param_value(value)?, false)?;
        Ok(())
    }
}
