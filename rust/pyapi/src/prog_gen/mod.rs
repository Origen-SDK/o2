//! Implements Python bindings for program generation data structures and functions

pub mod interface;
mod test;
mod test_invocation;

use origen::core::tester::TesterSource;
use origen::prog_gen::{flow_api, ParamType, ParamValue};
use origen::{Result, FLOW};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::wrap_pyfunction;
use std::thread;
pub use test::Test;
pub use test_invocation::TestInvocation;

#[pymodule]
/// Implements the module _origen.prog_gen in Python
pub fn prog_gen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(start_new_flow))?;
    m.add_wrapped(wrap_pyfunction!(end_flow))?;
    m.add_wrapped(wrap_pyfunction!(render))?;
    Ok(())
}

#[pyfunction]
fn start_new_flow(name: &str, sub_flow: Option<bool>) -> PyResult<usize> {
    let sub_flow = match sub_flow {
        None => false,
        Some(x) => x,
    };
    if sub_flow {
        Ok(flow_api::start_sub_flow(name, None)?)
    } else {
        FLOW.start(name)?;
        Ok(0)
    }
}

#[pyfunction]
fn end_flow(ref_id: usize, sub_flow: Option<bool>) -> PyResult<()> {
    let sub_flow = match sub_flow {
        None => false,
        Some(x) => x,
    };
    if sub_flow {
        Ok(flow_api::end_sub_flow(ref_id)?)
    } else {
        Ok(FLOW.end()?)
    }
}

// Called automatically by Origen once all test program source files have been executed
#[pyfunction]
fn render(py: Python) -> PyResult<Vec<String>> {
    let continue_on_fail = true;
    py.allow_threads(|| {
        let targets = {
            let tester = origen::tester();
            tester.targets().clone()
        };
        let threads: Vec<_> = targets.iter().enumerate().map(|(i, t)| {
            let t = t.to_owned();
            thread::spawn(move || {
                match t {
                    TesterSource::External(g) => {
                        log_error!("Python based tester targets are not supported for program generation yet, no action taken for target: {}", g);
                        Ok(vec![])
                    }
                    _ => {
                        let mut tester = origen::tester();
                        let files = tester.render_program_for_target_at(i, true);
                        match files {
                            Err(e) => {
                                let msg = e.to_string();
                                if continue_on_fail {
                                    origen::STATUS.inc_unhandled_error_count();
                                    log_error!("{}", &msg);
                                    Ok(vec![])
                                } else {
                                    Err(e)
                                }
                            }
                            Ok(paths) => Ok(paths)
                        }
                    }
                }
            })
        }).collect();
        let mut generated_files: Vec<String> = vec![];
        for thread in threads {
            match thread.join() {
                Err(_e) => log_error!("Something has gone wrong when doing the final program render"),
                Ok(v) => match v {
                    Err(e) => log_error!("{}", e),
                    Ok(paths) => {
                        for path in &paths {
                            generated_files.push(format!("{}", path.display()));
                        }
                    }
                }
            }
        }
        Ok(generated_files)
    })
}

fn to_param_value(value: &PyAny) -> Result<ParamValue> {
    Ok(if let Ok(v) = value.extract::<bool>() {
        ParamValue::Bool(v)
    } else if let Ok(v) = value.extract::<i64>() {
        ParamValue::Int(v)
    } else if let Ok(v) = value.extract::<u64>() {
        ParamValue::UInt(v)
    } else if let Ok(v) = value.extract::<f64>() {
        ParamValue::Float(v)
    } else if let Ok(v) = value.extract::<String>() {
        ParamValue::String(v)
    } else {
        ParamValue::Any(format!("{}", value.str()?))
    })
}

#[allow(dead_code)]
fn to_param_value_with_type(ptype: &ParamType, value: &PyAny) -> Result<ParamValue> {
    match ptype {
        ParamType::Bool => {
            if let Ok(v) = value.extract::<bool>() {
                Ok(ParamValue::Bool(v))
            } else {
                error!("Illegal value, expected a Boolean, got: '{}'", value)
            }
        }
        ParamType::Int => {
            if let Ok(v) = value.extract::<i64>() {
                Ok(ParamValue::Int(v))
            } else {
                error!("Illegal value, expected an Integer, got: '{}'", value)
            }
        }
        ParamType::UInt => {
            if let Ok(v) = value.extract::<u64>() {
                Ok(ParamValue::UInt(v))
            } else {
                error!(
                    "Illegal value, expected an Unsigned Integer, got: '{}'",
                    value
                )
            }
        }
        ParamType::Float => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Float(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Current => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Current(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Voltage => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Voltage(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Time => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Time(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Frequency => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Frequency(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::String => {
            if let Ok(v) = value.extract::<String>() {
                Ok(ParamValue::String(v))
            } else {
                error!("Illegal value, expected a String, got: '{}'", value)
            }
        }
        ParamType::Any => Ok(ParamValue::Any(format!("{}", value.str()?))),
    }
}
