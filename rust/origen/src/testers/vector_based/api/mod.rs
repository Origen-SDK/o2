use crate::generator::ast::{Node};
use crate::{Error, Result, node};
use std::collections::HashMap;
use indexmap::IndexMap;

// pub enum PinInputTypes {
//     PinId(usize), // Physical pin ID
//     PinGroupId(usize), // Pin group ID
//     PinName((String, usize)), // Physical pin name, model id
//     Pin
// }

pub fn cycle(compressable: bool) -> Result<Node> {
    Ok(node!(Cycle, 1, compressable))
}

pub fn repeat(count: u128, compressable: bool) -> Result<Node> {
    Ok(node!(Cycle, count as u32, compressable))
}

pub fn comment(message: &str) -> Result<Node> {
    Ok(node!(Text, message.to_string()))
}

// pub fn set_pin_drive_high(pin: &String) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     h.insert(pin.to_string(), (PinActions::DriveHigh, 1));
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_pin_drive_low(pin: &String) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     h.insert(pin.to_string(), (PinActions::DriveLow, 1));
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_pin_drive(pin: &String, val: bool) -> Result<Node> {
//     if val {
//         set_pin_drive_high(pin)
//     } else {
//         set_pin_drive_low(pin)
//     }
// }

// pub fn set_pin_verify_high(pin: &String) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     h.insert(pin.to_string(), (PinActions::VerifyHigh, 1));
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_pin_verify_low(pin: &String) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     h.insert(pin.to_string(), (PinActions::VerifyLow, 1));
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_pin_verify(pin: &String, val: bool) -> Result<Node> {
//     if val {
//         set_pin_verify_high(pin)
//     } else {
//         set_pin_verify_low(pin)
//     }
// }

// pub fn set_pin_highz(pin: &String) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     h.insert(pin.to_string(), (PinActions::HighZ, 1));
//     Ok(crate::node!(PinAction, h))
// }

// pub fn drive_pin_high(pin: &String) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_drive_high(pin)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn drive_pin_low(pin: &String) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_drive_low(pin)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn drive_pin(pin: &String, val: bool) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_drive(pin, val)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn verify_pin_high(pin: &String) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_verify_high(pin)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn verify_pin_low(pin: &String) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_verify_low(pin)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn verify_pin(pin: &String, val: bool) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_verify(pin, val)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn highz_pin(pin: &String) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_pin_highz(pin)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn set_drive_high(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::DriveHigh, 1)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_drive_low(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::DriveLow, 0)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_drive(pins: Vec<&String>, val: bool) -> Result<Node> {
//     if val {
//         set_drive_high(pins)
//     } else {
//         set_drive_low(pins)
//     }
// }

// pub fn set_verify_high(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::VerifyHigh, 1)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_verify_low(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::VerifyLow, 0)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_verify(pins: Vec<&String>, val: bool) -> Result<Node> {
//     if val {
//         set_verify_high(pins)
//     } else {
//         set_verify_low(pins)
//     }
// }

// pub fn set_capture(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::Capture, 1)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_highz(pins: Vec<&String>) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (PinActions::HighZ, 1)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn set_action(pins: Vec<&String>, action: PinActions) -> Result<Node> {
//     let mut h: HashMap<String, (PinActions, u8)> = HashMap::new();
//     pins.iter().for_each( |p| { h.insert((*p).to_string(), (action.clone(), 1)); });
//     Ok(crate::node!(PinAction, h))
// }

// pub fn drive_high(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_drive_high(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn drive_low(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_drive_low(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn drive(pins: Vec<&String>, val: bool) -> Result<Vec<Node>> {
//     if val {
//         drive_high(pins)
//     } else {
//         drive_low(pins)
//     }
// }

// pub fn verify_high(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_verify_high(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn verify_low(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_verify_low(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn verify(pins: Vec<&String>, val: bool) -> Result<Vec<Node>> {
//     if val {
//         verify_high(pins)
//     } else {
//         verify_low(pins)
//     }
// }

// pub fn capture(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_capture(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn highz(pins: Vec<&String>) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_highz(pins)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

// pub fn action(pins: Vec<&String>, action: PinActions) -> Result<Vec<Node>> {
//     let mut nodes: Vec<Node> = vec!();
//     nodes.push(set_action(pins, action)?);
//     nodes.push(cycle()?);
//     Ok(nodes)
// }

static DRIVE_HIGH_CHAR: &str = "1";
static DRIVE_LOW_CHAR: &str = "0";
static VERIFY_HIGH_CHAR: &str = "H";
static VERIFY_LOW_CHAR: &str = "L";
static HIGHZ_CHAR: &str = "Z";
static CAPTURE_CHAR: &str = "C";
// static UNKNOWN_CHAR: &String = &"X".to_string();

// pub fn drive_pin_low(pin: usize, grp_id: Option<usize>) -> Result<Node> {
//     Ok(drive_low(vec!(pin), grp_id)?.first().unwrap())
// }

pub fn drive_low(pins: &Vec<usize>, grp_id: Option<usize>) -> Result<Vec<Node>> {
    drive(pins, false, grp_id)
}

pub fn drive_high(pins: &Vec<usize>, grp_id: Option<usize>) -> Result<Vec<Node>> {
    drive(pins, true, grp_id)
}

/// Backend analog for the frontend's <block>.pin("name").drive()
/// This sets the pin(s) without triggering a cycle.
pub fn drive(pins: &Vec<usize>, data: bool, grp_id: Option<usize>) -> Result<Vec<Node>> {
    if data {
        action(pins, DRIVE_HIGH_CHAR, grp_id)
    } else {
        action(pins, DRIVE_LOW_CHAR, grp_id)
    }
}

/// Backend analog for the frontend's <block>.pin("name").drive().cycle()
pub fn drive_cycle(pins: &Vec<usize>, data: bool, grp_id: Option<usize>) -> Result<Vec<Node>> {
    //let mut nodes: Vec<Node> = vec!();
    //nodes.push(set_action(pins, action)?);
    let mut nodes = drive(pins, data, grp_id)?;
    nodes.push(cycle(true)?);
    Ok(nodes)
}

pub fn verify_high(pins: &Vec<usize>, grp_id: Option<usize>) -> Result<Vec<Node>> {
    verify(pins, true, grp_id)
}

pub fn verify_low(pins: &Vec<usize>, grp_id: Option<usize>) -> Result<Vec<Node>> {
    verify(pins, false, grp_id)
}

pub fn verify(pins: &Vec<usize>, data: bool, grp_id: Option<usize>) -> Result<Vec<Node>> {
    if data {
        action(pins, VERIFY_HIGH_CHAR, grp_id)
    } else {
        action(pins, VERIFY_LOW_CHAR, grp_id)
    }
}


pub fn highz(pins: &Vec<usize>, grp_id: Option<usize>) -> Result<Vec<Node>> {
    action(pins, HIGHZ_CHAR, grp_id)
}

pub fn pin_action_map() -> IndexMap<usize, (String, Option<HashMap<String, crate::Metadata>>)> {
    IndexMap::new()
}

pub fn action(pins: &Vec<usize>, action: &str, grp_id: Option<usize>) -> Result<Vec<Node>> {
    // let mut nodes: Vec<Node> = vec!();

    let nodes: Vec<Node> = pins.iter().map( |id| node!(PinAction, *id, action.to_string(), None) ).collect();

    if let Some(id) = grp_id {
        let mut n = node!(PinGroupAction, id, vec![action.to_string(); pins.len()], None);
        n.add_children(nodes);
        Ok(vec!(n))
    } else {
        Ok(nodes)
    }

    // let map = pin_action_map();
    // for p_id in pins {
    //     nodes.push(node!(PinAction, p_id, action.to_string(), {
    //         if let Some(id) = grp_id {
    //             Some((grp_id, None))
    //         } else {
    //             None
    //         }
    //     }));
    // }
    // Ok(nodes)
}

pub fn push_data(
    // pins: Vec<&String>,
    pins: &IndexMap<usize, Vec<usize>>,
    data: &impl num::ToPrimitive,
    data_width: usize,
    lsb_first: bool
) -> Result<Vec<Node>> {
    drive_data(pins, data, data_width, lsb_first, None::<&u8>, None)
}

pub fn drive_data(
    // pins: Vec<&String>,
    pins: &IndexMap<usize, Vec<usize>>,
    data: &impl num::ToPrimitive, // this will later be casted to a u128, so can accept negatives here
    data_width: usize,
    lsb_first: bool,
    _overlays: Option<&impl num::ToPrimitive>,
    _overlay_str: Option<&String>
) -> Result<Vec<Node>> {
    let data_bits = to_bits(data, data_width, lsb_first)?;

    let mut pin_states: Vec<Node> = vec!();

    // flatten the pin groups
    let mut bus: Vec<(usize, usize)> = vec![];
    for (grp_id, pin_ids) in pins.iter() {
        for pin_id in pin_ids.iter() {
            bus.push((*grp_id, *pin_id));
        }
    }

    for (_chunk_idx, chunk) in data_bits.chunks(bus.len()).enumerate() {
        // let mut this_cycle: HashMap<String, (PinActions, u8)> = HashMap::new();
        let mut this_cycle: Vec<Node> = vec![];
        let mut current_grp = bus[0].0;
        let mut this_grp_nodes: Vec<Node> = vec![];
        let mut this_grp_data = vec![];

        for (pos, bit) in chunk.iter().enumerate() {
            if current_grp != bus[pos].0 {
                // Push this group and move on to the next
                current_grp = bus[pos].0;
                let mut n =node!(PinGroupAction, current_grp, this_grp_data.clone(), None);
                n.add_children(this_grp_nodes);
                this_cycle.push(n);
                this_grp_nodes = vec![];
            }

            if *bit {
                // this_cycle.insert(bus[pos].to_string(), (PinActions::DriveHigh, 1));
                this_grp_nodes.push(node!(PinAction, bus[pos].1, DRIVE_HIGH_CHAR.to_string(), None));
                this_grp_data.push(DRIVE_HIGH_CHAR.to_string());
            } else {
                this_grp_nodes.push(node!(PinAction, bus[pos].1, DRIVE_LOW_CHAR.to_string(), None));
                this_grp_data.push(DRIVE_LOW_CHAR.to_string());
                // this_cycle.insert(bus[pos].to_string(), (PinActions::DriveLow, 0));
            }
        }
        // pin_states.push(crate::node!(PinAction, this_cycle));
        // pin_states.push(cycle(true)?);
        let mut n =node!(PinGroupAction, current_grp, this_grp_data, None);
        n.add_children(this_grp_nodes);
        this_cycle.push(n);
        pin_states.append(&mut this_cycle);
        pin_states.push(cycle(true)?);
    }
    Ok(pin_states)
}

pub fn verify_data(
    // pins: Vec<&String>,
    pins: &IndexMap<usize, Vec<usize>>,
    data: &impl num::ToPrimitive,
    data_width: usize,
    lsb_first: bool
) -> Result<Vec<Node>> {
    read_data(pins, data, data_width, lsb_first, None::<&u8>, None::<&u8>)
}

pub fn read_data(
    pins: &IndexMap<usize, Vec<usize>>,
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

    // flatten the pin groups
    let mut bus: Vec<(usize, usize)> = vec![];
    for (grp_id, pin_ids) in pins.iter() {
        for pin_id in pin_ids.iter() {
            bus.push((*grp_id, *pin_id));
        }
    }

    for (idx, chunk) in data_bits.chunks(bus.len()).enumerate() {
        // let mut this_cycle: HashMap<String, (PinActions, u8)> = HashMap::new();
        let mut this_cycle: Vec<Node> = vec![];
        let mut current_grp = bus[0].0;
        let mut this_grp_nodes: Vec<Node> = vec![];
        let mut this_grp_data = vec![];
        // let mut n = node!(PinGroupAction, current_grp_id, vec![action.to_string(); pins.len()], None);
        for (pos, bit) in chunk.iter().enumerate() {
            if current_grp != bus[pos].0 {
                // Push this group and move on to the next
                current_grp = bus[pos].0;
                let mut n =node!(PinGroupAction, current_grp, this_grp_data.clone(), None);
                n.add_children(this_grp_nodes);
                this_cycle.push(n);
                this_grp_nodes = vec![];
            }
            if should_verify && verify_bits[(idx*bus.len()) + pos] {
                if *bit {
                    // this_cycle.insert(pins[pos].to_string(), (PinActions::VerifyHigh, 1));
                    this_grp_nodes.push(node!(PinAction, bus[pos].1, VERIFY_HIGH_CHAR.to_string(), None));
                    this_grp_data.push(VERIFY_HIGH_CHAR.to_string());
                    // n.add_children(nodes);
                    // Ok(vec!(n))
                } else {
                    // this_cycle.insert(pins[pos].to_string(), (PinActions::VerifyLow, 0));
                    this_grp_nodes.push(node!(PinAction, bus[pos].1, VERIFY_LOW_CHAR.to_string(), None));
                    this_grp_data.push(VERIFY_LOW_CHAR.to_string());
                }
            } else if should_capture && capture_bits[(idx*pins.len()) + pos] {
                this_grp_nodes.push(node!(PinAction, bus[pos].1, CAPTURE_CHAR.to_string(), None));
                this_grp_data.push(CAPTURE_CHAR.to_string());
                // this_cycle.insert(pins[pos].to_string(), (PinActions::Capture, 0));
            } else {
                this_grp_nodes.push(node!(PinAction, bus[pos].1, HIGHZ_CHAR.to_string(), None));
                this_grp_data.push(HIGHZ_CHAR.to_string());
                // this_cycle.insert(pins[pos].to_string(), (PinActions::HighZ, 0));
            }
        }
        // Push this last group to the current cycle, push the cycle updates, then finally cycle the tester
        let mut n =node!(PinGroupAction, current_grp, this_grp_data, None);
        n.add_children(this_grp_nodes);
        this_cycle.push(n);
        pin_states.append(&mut this_cycle);
        pin_states.push(cycle(true)?);

        // pin_states.push(crate::node!(PinAction, this_cycle));
        // pin_states.push(cycle(true)?);
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
