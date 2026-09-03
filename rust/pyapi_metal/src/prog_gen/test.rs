use super::src_caller_meta;
use super::TestCollectionItem;
use super::{to_limit_param_value, to_param_value};
use crate::prog_gen::flow_options;
use origen_metal::prog_gen::{flow_api, Limit, LimitSelector, ParamValue, SupportedTester};
use origen_metal::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

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

    pub fn _set_attr(
        &self,
        name: &str,
        value: Option<ParamValue>,
        allow_missing: bool,
    ) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, allow_missing, src_caller_meta())?;
        Ok(())
    }
}

#[pymethods]
impl Test {
    #[setter]
    pub fn set_lo_limit(&self, value: &PyAny) -> PyResult<()> {
        let value = match to_limit_param_value(value)? {
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
        let value = match to_limit_param_value(value)? {
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

    #[pyo3(signature=(name, number=None, lo=None, hi=None))]
    pub fn add_limit(
        &self,
        name: String,
        number: Option<usize>,
        lo: Option<&PyAny>,
        hi: Option<&PyAny>,
    ) -> PyResult<()> {
        let lo = match lo {
            Some(value) => to_limit_param_value(value)?.map(|value| Limit {
                kind: origen_metal::prog_gen::LimitType::GTE,
                value,
                unit: None,
            }),
            None => None,
        };
        let hi = match hi {
            Some(value) => to_limit_param_value(value)?.map(|value| Limit {
                kind: origen_metal::prog_gen::LimitType::LTE,
                value,
                unit: None,
            }),
            None => None,
        };
        flow_api::define_sub_test(self.id, name, number, lo, hi, src_caller_meta())?;
        Ok(())
    }

    fn set_measure_mode(&self, mode: String) -> PyResult<()> {
        let value = match mode.to_ascii_lowercase().as_str() {
            "current" | "fvmi" => 2,
            "voltage" | "fimv" => 1,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown UltraFLEX measure mode '{}'",
                    mode
                )))
            }
        };
        self._set_attr("measure_mode", Some(ParamValue::Int(value)), false)?;
        Ok(())
    }

    #[pyo3(signature=(*flags))]
    fn set_wait_flags(&self, flags: &PyTuple) -> PyResult<()> {
        let mut clean = vec![];
        for flag in flags {
            let flag = flag.extract::<String>()?.to_ascii_lowercase();
            if !matches!(flag.as_str(), "a" | "b" | "c" | "d") {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown UltraFLEX wait flag '{}'",
                    flag
                )));
            }
            clean.push(flag);
        }
        flow_api::set_wait_flags(self.id, clean, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(collection_name, instance_id, allow_missing=false))]
    pub fn add_collection_item(
        &self,
        collection_name: &str,
        instance_id: &str,
        allow_missing: bool,
    ) -> Result<TestCollectionItem> {
        let id = flow_api::define_test_collection_item(
            self.id,
            collection_name,
            instance_id,
            allow_missing,
            src_caller_meta(),
        )?;
        Ok(TestCollectionItem::new(id))
    }

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        self._set_attr(name, to_param_value(value)?, false)?;
        Ok(())
    }
}
