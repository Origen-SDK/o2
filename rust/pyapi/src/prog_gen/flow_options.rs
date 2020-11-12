//! Functions concerned with extracting known values from kwargs

use origen::prog_gen::FlowID;
use origen::Result;
use pyo3::types::PyDict;

pub fn is_flow_option(key: &str) -> bool {
    match key {
        "id" => true,
        "bin" => true,
        "softbin" => true,
        "soft_bin" => true,
        _ => false,
    }
}

/// Returns a FlowID object from an "id" field present in the args, or else
/// a generated ID.
pub fn get_flow_id(kwargs: Option<&PyDict>) -> Result<FlowID> {
    if let Some(kwargs) = kwargs {
        if let Some(id) = kwargs.get_item("id") {
            if let Ok(v) = id.extract::<String>() {
                return Ok(FlowID::from_str(&v));
            } else if let Ok(v) = id.extract::<usize>() {
                return Ok(FlowID::from_int(v));
            } else {
                return error!(
                    "Illegal 'id' value, expected a String or an Integer, got: '{}'",
                    id
                );
            }
        }
    }
    Ok(FlowID::new())
}

pub fn get_bin(kwargs: Option<&PyDict>) -> Result<Option<usize>> {
    if let Some(kwargs) = kwargs {
        if let Some(bin) = kwargs.get_item("bin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!("Illegal 'bin' value, expected an Integer, got: '{}'", bin);
            }
        }
    }
    Ok(None)
}

pub fn get_softbin(kwargs: Option<&PyDict>) -> Result<Option<usize>> {
    if let Some(kwargs) = kwargs {
        if let Some(bin) = kwargs.get_item("softbin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!(
                    "Illegal 'softbin' value, expected an Integer, got: '{}'",
                    bin
                );
            }
        }
        if let Some(bin) = kwargs.get_item("soft_bin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!(
                    "Illegal 'soft_bin' value, expected an Integer, got: '{}'",
                    bin
                );
            }
        }
    }
    Ok(None)
}
