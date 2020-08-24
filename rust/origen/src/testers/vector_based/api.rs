use crate::core::model::pins::pin::{PinActions};
use crate::generator::ast::{Node};
use crate::{Error, Result, node};
use std::collections::HashMap;

pub fn cycle() -> Result<Node> {
    Ok(node!(Cycle, 1, true))
}

pub fn repeat(count: u128) -> Result<Node> {
    Ok(node!(Cycle, count as u32, true))
}

pub fn comment(message: &str) -> Result<Node> {
    Ok(node!(Text, message.to_string()))
}

pub fn set_pin_drive_high(pin: &String) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    h.insert(pin.to_string(), (PinActions::DriveHigh, 1));
    Ok(crate::node!(PinAction, h))
}

pub fn set_pin_drive_low(pin: &String) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    h.insert(pin.to_string(), (PinActions::DriveLow, 1));
    Ok(crate::node!(PinAction, h))
}

pub fn set_pin_drive(pin: &String, val: bool) -> Result<Node> {
    if val {
        set_pin_drive_high(pin)
    } else {
        set_pin_drive_low(pin)
    }
}

pub fn set_pin_verify_high(pin: &String) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    h.insert(pin.to_string(), (PinActions::VerifyHigh, 1));
    Ok(crate::node!(PinAction, h))
}

pub fn set_pin_verify_low(pin: &String) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    h.insert(pin.to_string(), (PinActions::VerifyLow, 1));
    Ok(crate::node!(PinAction, h))
}

pub fn set_pin_verify(pin: &String, val: bool) -> Result<Node> {
    if val {
        set_pin_verify_high(pin)
    } else {
        set_pin_verify_low(pin)
    }
}

pub fn set_pin_highz(pin: &String) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    h.insert(pin.to_string(), (PinActions::HighZ, 1));
    Ok(crate::node!(PinAction, h))
}

pub fn drive_pin_high(pin: &String) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_drive_high(pin)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn drive_pin_low(pin: &String) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_drive_low(pin)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn drive_pin(pin: &String, val: bool) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_drive(pin, val)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn verify_pin_high(pin: &String) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_verify_high(pin)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn verify_pin_low(pin: &String) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_verify_low(pin)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn verify_pin(pin: &String, val: bool) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_verify(pin, val)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn highz_pin(pin: &String) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_pin_highz(pin)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn set_drive_high(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::DriveHigh, 1)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_drive_low(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::DriveLow, 0)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_drive(pins: Vec<&String>, val: bool) -> Result<Node> {
    if val {
        set_drive_high(pins)
    } else {
        set_drive_low(pins)
    }
}

pub fn set_verify_high(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::VerifyHigh, 1)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_verify_low(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::VerifyLow, 0)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_verify(pins: Vec<&String>, val: bool) -> Result<Node> {
    if val {
        set_verify_high(pins)
    } else {
        set_verify_low(pins)
    }
}

pub fn set_capture(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::Capture, 1)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_highz(pins: Vec<&String>) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::HighZ, 1)); });
    Ok(crate::node!(PinAction, h))
}

pub fn set_action(pins: Vec<&String>, action: PinActions) -> Result<Node> {
    let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
    pins.iter().for_each( |p| { h.insert((*p).to_string(), (action.clone(), 1)); });
    Ok(crate::node!(PinAction, h))
}

pub fn drive_high(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_drive_high(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn drive_low(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_drive_low(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn drive(pins: Vec<&String>, val: bool) -> Result<Vec<Node>> {
    if val {
        drive_high(pins)
    } else {
        drive_low(pins)
    }
}

pub fn verify_high(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_verify_high(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn verify_low(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_verify_low(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn verify(pins: Vec<&String>, val: bool) -> Result<Vec<Node>> {
    if val {
        verify_high(pins)
    } else {
        verify_low(pins)
    }
}

pub fn capture(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_capture(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn highz(pins: Vec<&String>) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_highz(pins)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn action(pins: Vec<&String>, action: PinActions) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = vec!();
    nodes.push(set_action(pins, action)?);
    nodes.push(cycle()?);
    Ok(nodes)
}

pub fn push_data(
    pins: Vec<&String>,
    data: &impl num::ToPrimitive,
    data_width: usize,
    lsb_first: bool
) -> Result<Vec<Node>> {
    drive_data(pins, data, data_width, lsb_first, None::<&u8>, None)
}

pub fn drive_data(
    pins: Vec<&String>,
    data: &impl num::ToPrimitive, // this will later be casted to a u128, so can accept negatives here
    data_width: usize,
    lsb_first: bool,
    overlays: Option<&impl num::ToPrimitive>,
    overlay_str: Option<&String>
) -> Result<Vec<Node>> {
    let data_bits = to_bits(data, data_width, lsb_first)?;

    let mut pin_states: Vec<Node> = vec!();
    for (chunk_idx, chunk) in data_bits.chunks(pins.len()).enumerate() {
        let mut this_cycle: HashMap<String, (PinActions, u8)> = HashMap::new();
        for (pos, bit) in chunk.iter().enumerate() {
            if *bit {
                this_cycle.insert(pins[pos].to_string(), (PinActions::DriveHigh, 1));
            } else {
                this_cycle.insert(pins[pos].to_string(), (PinActions::DriveLow, 0));
            }
        }
        pin_states.push(crate::node!(PinAction, this_cycle));
        pin_states.push(crate::cycle!(1));
    }
    Ok(pin_states)
}

pub fn verify_data(
    pins: Vec<&String>,
    data: &impl num::ToPrimitive,
    data_width: usize,
    lsb_first: bool
) -> Result<Vec<Node>> {
    read_data(pins, data, data_width, lsb_first, None::<&u8>, None::<&u8>)
}

/// verify_data! explicitly verifies (i.e., H/L) the given data on the given pins.
/// read_data! is general purpose and encompasses both verifying and capturing.
pub fn read_data(
    pins: Vec<&String>,
    data: &impl num::ToPrimitive, // this will later be casted to a u128, so can accept negatives here
    data_width: usize,
    lsb_first: bool,
    verifies: Option<&impl num::ToPrimitive>, // Only makes sense to have unsigned data here
    captures: Option<&impl num::ToPrimitive> // Only makes sense to have unsigned data here
) -> Result<Vec<Node>> {
    let mut pin_states: Vec<crate::generator::ast::Node> = vec!();
    let data_bits = to_bits(data, data_width, lsb_first)?;
    let verify_bits;
    let capture_bits;
    let should_verify;
    let should_capture;

    // I don't know why, but I couldn't get this to compile using any other syntax
    // Had to be a if-else
    //   - coreyeng
    if verifies.is_some() {
        verify_bits = to_bits(verifies.unwrap(), data_width, lsb_first)?;
        should_verify = true;
    } else {
        verify_bits = to_bits(&0, data_width, lsb_first)?;
        should_verify = false;
    }
    if captures.is_some() {
        capture_bits = to_bits(captures.unwrap(), data_width, lsb_first)?;
        should_capture = true;
    } else {
        capture_bits = to_bits(&0, data_width, lsb_first)?;
        should_capture = false;
    }

    for (idx, chunk) in data_bits.chunks(pins.len()).enumerate() {
        let mut this_cycle: HashMap<String, (PinActions, u8)> = HashMap::new();
        for (pos, bit) in chunk.iter().enumerate() {
            if should_verify && verify_bits[(idx*pins.len()) + pos] {
                if *bit {
                    this_cycle.insert(pins[pos].to_string(), (PinActions::VerifyHigh, 1));
                } else {
                    this_cycle.insert(pins[pos].to_string(), (PinActions::VerifyLow, 0));
                }
            } else if should_capture && capture_bits[(idx*pins.len()) + pos] {
                this_cycle.insert(pins[pos].to_string(), (PinActions::Capture, 0));
            } else {
                this_cycle.insert(pins[pos].to_string(), (PinActions::HighZ, 0));
            }
        }
        pin_states.push(crate::node!(PinAction, this_cycle));
        pin_states.push(crate::cycle!(1));
    }

    Ok(pin_states)
}

// Todo: Add some error handing/checking to this.
/// Creates a Vec<bool> from the bit values in `data` of size `data_width`.
pub fn to_bits(data: &impl num::ToPrimitive, data_width: usize, lsb_first: bool) -> Result<Vec<bool>> {
    let mut bits: Vec<bool> = Vec::with_capacity(data_width);
    match data.to_u128() {
        Some(d) => {
            for i in 0..data_width {
                if ((d >> i) & 1) == 1 {
                    bits.push(true);
                } else {
                    bits.push(false);
                }
            }
            if !lsb_first {
                bits.reverse();
            }
            Ok(bits)
        },
        None => Err(Error::new("Could not convert 'data' to u128"))
    }
}
