pub mod tester_apis;
mod test_invocation;
mod flow_options;
mod test;
mod group;
mod pattern_group;
mod condition;
mod resources;
pub mod interface;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use test_invocation::TestInvocation;
use test::Test;
use group::Group;
use pattern_group::PatternGroup;
use condition::Condition;
use resources::Resources;

use origen_metal::ast::Meta;
use pyo3::types::PyAny;
use origen_metal::{Result, Error, FLOW};
use origen_metal::prog_gen::{PGM, ParamType, ParamValue};
use pyo3::prelude::*;
use origen_metal::prog_gen::{flow_api, FlowCondition, SupportedTester};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "prog_gen")?;
    subm.add_wrapped(wrap_pyfunction!(start_new_flow))?;
    subm.add_wrapped(wrap_pyfunction!(end_flow))?;
    subm.add_wrapped(wrap_pyfunction!(render_program_for))?;
    subm.add_wrapped(wrap_pyfunction!(start_eq_block))?;
    subm.add_wrapped(wrap_pyfunction!(end_eq_block))?;
    subm.add_wrapped(wrap_pyfunction!(start_neq_block))?;
    subm.add_wrapped(wrap_pyfunction!(end_neq_block))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
fn render_program_for(tester: &str, output_dir: &str) -> PyResult<Vec<PathBuf>> {
    let t = match origen_metal::prog_gen::SupportedTester::from_str(tester) {
        Ok(t) => t,
        Err(e) => {
            return Err(PyErr::from(Error::new(&format!(
                "Failed to identify a supported tester type from '{}': {}",
                tester, e
            ))))
        }
    };
    let output_dir = Path::new(output_dir).to_path_buf();
    let r = origen_metal::prog_gen::render_program(t, &output_dir)?;
    Ok(r.0)
}

#[pyfunction]
fn start_eq_block(testers: Vec<&str>) -> PyResult<(usize, Vec<String>)> {
    let mut ts: Vec<SupportedTester> = vec![];
    let mut clean_testers: Vec<String> = vec![];
    for t in testers {
        let st = SupportedTester::new(t)?;
        clean_testers.push(st.to_string());
        ts.push(st);
    }
    let n = node!(PGM::TesterEq, ts);
    let ref_id = FLOW.push_and_open(n)?;
    Ok((ref_id, clean_testers))
}

#[pyfunction]
fn end_eq_block(ref_id: usize) -> PyResult<()> {
    FLOW.close(ref_id)?;
    Ok(())
}

#[pyfunction]
fn start_neq_block(testers: Vec<&str>) -> PyResult<usize> {
    let mut ts: Vec<SupportedTester> = vec![];
    for t in testers {
        let st = SupportedTester::new(t)?;
        ts.push(st);
    }
    let n = node!(PGM::TesterNeq, ts);
    let ref_id = FLOW.push_and_open(n)?;
    Ok(ref_id)
}

#[pyfunction]
fn end_neq_block(ref_id: usize) -> PyResult<()> {
    FLOW.close(ref_id)?;
    Ok(())
}

#[pyfunction]
fn start_new_flow(
    name: &str,
    sub_flow: Option<bool>,
    bypass_sub_flows: Option<bool>,
    add_flow_enable: Option<&str>,
) -> PyResult<Vec<usize>> {
    let sub_flow = match sub_flow {
        None => false,
        Some(x) => x,
    };
    let mut refs = vec![];
    if sub_flow {
        refs.push(flow_api::start_sub_flow(name, None, None)?);
    } else {
        FLOW.start(name)?;
        refs.push(0);
        if let Some(bypass) = bypass_sub_flows {
            if bypass {
                refs.push(flow_api::start_bypass_sub_flows(None)?);
            }
        }
        if let Some(enable) = add_flow_enable {
            let flag = format!("{}_enable", name);
            refs.push(flow_api::start_condition(
                FlowCondition::IfEnable(vec![flag.clone()]),
                None,
            )?);
            if enable.to_lowercase() == "enabled" {
                flow_api::set_default_flag_state(flag, true, None)?;
            } else if enable.to_lowercase() == "disabled" {
                flow_api::set_default_flag_state(flag, false, None)?;
            } else {
                return Err(PyErr::from(Error::new(&format!(
                    "The add_flow_enable argument must be either None (default), \"enabled\" or \"disabled\", got '{}'",
                    enable
                ))));
            }
        }
    }
    refs.reverse();
    Ok(refs)
}

#[pyfunction]
fn end_flow(ref_ids: Vec<usize>) -> PyResult<()> {
    for ref_id in ref_ids {
        if ref_id == 0 {
            FLOW.end()?;
        } else {
            flow_api::end_block(ref_id)?;
        }
    }
    Ok(())
}

pub fn src_caller_meta() -> Option<Meta> {
    None
}

pub fn to_param_value(value: &PyAny) -> Result<Option<ParamValue>> {
    Ok(if let Ok(v) = value.extract::<bool>() {
        Some(ParamValue::Bool(v))
    } else if let Ok(v) = value.extract::<u64>() {
        Some(ParamValue::UInt(v))
    } else if let Ok(v) = value.extract::<i64>() {
        Some(ParamValue::Int(v))
    } else if let Ok(v) = value.extract::<f64>() {
        Some(ParamValue::Float(v))
    } else if let Ok(v) = value.extract::<String>() {
        Some(ParamValue::String(v))
    } else if let Ok(None) = value.extract::<Option<String>>() {
        None
    } else {
        Some(ParamValue::Any(format!("{}", value.str()?)))
    })
}

#[allow(dead_code)] // Could be used in future
pub fn to_param_value_with_type(ptype: &ParamType, value: &PyAny) -> Result<ParamValue> {
    match ptype {
        ParamType::Bool => {
            if let Ok(v) = value.extract::<bool>() {
                Ok(ParamValue::Bool(v))
            } else {
                bail!("Illegal value, expected a Boolean, got: '{}'", value)
            }
        }
        ParamType::Int => {
            if let Ok(v) = value.extract::<i64>() {
                Ok(ParamValue::Int(v))
            } else {
                bail!("Illegal value, expected an Integer, got: '{}'", value)
            }
        }
        ParamType::UInt => {
            if let Ok(v) = value.extract::<u64>() {
                Ok(ParamValue::UInt(v))
            } else {
                bail!(
                    "Illegal value, expected an Unsigned Integer, got: '{}'",
                    value
                )
            }
        }
        ParamType::Float => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Float(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Current => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Current(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Voltage => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Voltage(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Time => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Time(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Frequency => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Frequency(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::String => {
            if let Ok(v) = value.extract::<String>() {
                Ok(ParamValue::String(v))
            } else {
                bail!("Illegal value, expected a String, got: '{}'", value)
            }
        }
        ParamType::Any => Ok(ParamValue::Any(format!("{}", value.str()?))),
    }
}
