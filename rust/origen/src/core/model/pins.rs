pub mod pin;
pub mod pin_group;
pub mod pin_header;
pub mod pin_store;
use super::super::dut::Dut;
use crate::generator::PAT;
use crate::standards::actions::*;
use crate::testers::vector_based::api::{cycle, repeat, repeat2, repeat2_node};
use crate::{Result, Transaction, TEST};
use origen_metal::ast::Node;

use regex::Regex;

use super::Model;
use indexmap::IndexMap;
use pin::{Pin, PinAction, ResolvePinActions};
use pin_group::PinGroup;
use pin_store::PinStore;

pub type PinGroupIdendifer = (usize, String);

#[derive(Debug, Clone)]
pub struct PinCollection<'a> {
    grp_ids: Option<Vec<(usize, usize)>>,
    pins: Vec<&'a Pin>,
}

impl<'a> PinCollection<'a> {
    pub fn from_group(dut: &'a crate::Dut, grp_name: &str, model_id: usize) -> crate::Result<Self> {
        let pins = dut._resolve_to_flattened_pins(&vec![(model_id, grp_name.to_string())])?;
        Ok(Self {
            grp_ids: Some(vec![(
                dut._get_pin_group(model_id, grp_name)?.id,
                pins.len(),
            )]),
            pins: pins,
        })
    }

    pub fn from_pin_store(dut: &'a crate::Dut, ps: &PinStore) -> crate::Result<Self> {
        Ok(Self {
            grp_ids: None,
            pins: ps.pin_ids.iter().map(|id| &dut.pins[*id]).collect(),
        })
    }

    pub fn from_pin_group(dut: &'a crate::Dut, grp: &PinGroup) -> crate::Result<Self> {
        Ok(Self {
            grp_ids: Some(vec![(grp.id, grp.pin_ids.len())]),
            pins: grp.pin_ids.iter().map(|id| &dut.pins[*id]).collect(),
        })
    }

    // Haven't actually used this yet, so commenting it out until its needed in case it needs more work - coreyeng
    // pub fn from_groups(dut: &crate::Dut, grps: Vec<(usize, &str)>) -> crate::Result<Self> {
    //     let mut grp_ids = vec!();
    //     let mut p_ids: Vec<usize> = vec!();
    //     for grp in grps.iter() {
    //         let mut ids: Vec<usize> = dut._resolve_group_to_physical_pins(
    //             grp.0,
    //             grp.1
    //         )?.iter().map( |p| p.id).collect();
    //         grp_ids.push((dut._get_pin_group(grp.0, grp.1)?.id, ids.len()));
    //         p_ids.append(&mut ids);
    //     }
    //     let mut temp = p_ids.clone();
    //     temp.sort();
    //     temp.dedup();
    //     if p_ids.len() != temp.len() {
    //         bail!(
    //             "Duplicate physical pins detected when creating PinBus from {:?} - (resolved pin IDs: {:?}, unique pin IDs {:?})",
    //             grps,
    //             p_ids,
    //             temp
    //         )
    //     }
    //     Ok(Self {
    //         grp_ids: Some(grp_ids),
    //         pin_ids: p_ids
    //     })
    // }

    pub fn pin_names(&self) -> Vec<String> {
        self.pins.iter().map(|p| p.name.to_string()).collect()
    }

    pub fn width(&self) -> usize {
        self.pins.len()
    }

    pub fn contains_group_identifier(
        &self,
        dut: &Dut,
        id: PinGroupIdendifer,
    ) -> crate::Result<bool> {
        // A pin collection contains a pin group if all of the pin_ids in the
        // group are present and adjacent to one another. For example, if the
        // pin ids for a group are [0, 1, 2], then a collection of:
        // [3, 0, 1, 2, 4, 5] would contain pin group, but a collection of
        // [0, 3, 1, 4, 2, 5] would not, even though all the pins are present.
        let grp = dut._get_pin_group(id.0, &id.1)?;
        Ok(self.contains(&grp.pin_ids))
    }

    pub fn as_ids(&self) -> Vec<usize> {
        self.pins.iter().map(|p| p.id).collect()
    }

    fn contains(&self, query_ids: &Vec<usize>) -> bool {
        let p_ids = self.as_ids();
        if query_ids.len() == 0 {
            return false;
        }

        if let Some(pos) = p_ids.iter().position(|&i| i == *query_ids.first().unwrap()) {
            for (idx, id) in query_ids[1..].iter().enumerate() {
                if *id != p_ids[pos + idx] {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Applies the drive-high symbol to all the pins on this bus and pushes them to the AST
    pub fn drive_high(&self) -> &Self {
        TEST.append(&mut self.drive_high_nodes());
        &self
    }

    /// Applies the drive-high symbol to all the pins on this bus and returns the nodes without pushing them to the AST
    pub fn drive_high_nodes(&self) -> Vec<Node<PAT>> {
        self.drive_nodes(true)
    }

    /// Identical to "drive_high" except uses the drive-low symbol instead
    pub fn drive_low(&self) -> &Self {
        TEST.append(&mut self.drive_low_nodes());
        &self
    }

    /// Identical to "drive_low_nodes" except uses the drive-low symbol instead
    pub fn drive_low_nodes(&self) -> Vec<Node<PAT>> {
        self.drive_nodes(false)
    }

    /// Drives all pins to either the drive-high character, if 'state' is true,
    /// or the drive-low character, if the 'state is false
    pub fn drive(&self, state: bool) -> &Self {
        TEST.append(&mut self.drive_nodes(state));
        &self
    }

    pub fn drive_nodes(&self, state: bool) -> Vec<Node<PAT>> {
        if state {
            self.set_action_nodes(DRIVE_HIGH)
        } else {
            self.set_action_nodes(DRIVE_LOW)
        }
    }

    pub fn verify_high(&self) -> &Self {
        TEST.append(&mut self.verify_high_nodes());
        &self
    }

    pub fn verify_high_nodes(&self) -> Vec<Node<PAT>> {
        self.verify_nodes(true)
    }

    pub fn verify_low(&self) -> &Self {
        TEST.append(&mut self.verify_low_nodes());
        &self
    }

    pub fn verify_low_nodes(&self) -> Vec<Node<PAT>> {
        self.verify_nodes(false)
    }

    pub fn verify(&self, state: bool) -> &Self {
        TEST.append(&mut self.verify_nodes(state));
        &self
    }

    pub fn verify_nodes(&self, state: bool) -> Vec<Node<PAT>> {
        if state {
            self.set_action_nodes(VERIFY_HIGH)
        } else {
            self.set_action_nodes(VERIFY_LOW)
        }
    }

    pub fn capture(&self) -> &Self {
        TEST.append(&mut self.capture_nodes());
        &self
    }

    pub fn capture_nodes(&self) -> Vec<Node<PAT>> {
        self.set_action_nodes(CAPTURE)
    }

    pub fn highz(&self) -> &Self {
        TEST.append(&mut self.highz_nodes());
        &self
    }

    pub fn highz_nodes(&self) -> Vec<Node<PAT>> {
        self.set_action_nodes(HIGHZ)
    }

    /// Sets all the pins in this bus to an arbitrary action, pushing the nodes onto the AST
    pub fn set_action(&self, action: &str) -> &Self {
        TEST.append(&mut self.set_action_nodes(action));
        &self
    }

    pub fn set_actions<T: AsRef<str> + std::fmt::Display>(
        &self,
        actions: &Vec<T>,
    ) -> crate::Result<&Self> {
        if actions.len() != self.pins.len() {
            bail!(
                "Error in PinCollection (set_actions): Expected length of actions ({}) to equal length of pin collection ({})",
                actions.len(),
                self.pins.len()
            );
        }
        for (i, a) in actions.iter().enumerate() {
            let p = &self.pins[i];
            let mut paction = p.action.write().unwrap();
            *paction = PinAction::new(a);
            TEST.push(node!(PAT::PinAction, p.id, a.to_string(), None));
        }

        Ok(&self)
    }

    /// Sets all the pins in this bus to an arbitrary action, returning the nodes without pushing to the AST
    pub fn set_action_nodes(&self, action: &str) -> Vec<Node<PAT>> {
        if let Some(grps) = &self.grp_ids {
            let mut retn = vec![];
            let mut pin_ids_offset = 0;
            for (_i, grp) in grps.iter().enumerate() {
                let mut grp_node = node!(
                    PAT::PinGroupAction,
                    grp.0,
                    vec![action.to_string(); self.pins.len()],
                    None
                );
                grp_node.add_children(
                    (0..grp.1)
                        .map(|pin_i| {
                            let p = &self.pins[pin_ids_offset + pin_i];
                            let mut paction = p.action.write().unwrap();
                            *paction = PinAction::new(action);

                            node!(PAT::PinAction, p.id, action.to_string(), None)
                        })
                        .collect(),
                );
                retn.push(grp_node);
                pin_ids_offset += grp.1;
            }
            retn
        } else {
            self.pins
                .iter()
                .map(|p| {
                    let mut paction = p.action.write().unwrap();
                    *paction = PinAction::new(action);
                    node!(PAT::PinAction, p.id, action.to_string(), None)
                })
                .collect()
        }
    }

    /// Sets the pin actions per the given transaction
    /// Differs from "push_transaction" in that this is verified to fit within a single
    /// pin-action update. That is, the transaction width should match the PinCollection width exactly
    pub fn set_from_transaction(&self, trans: &Transaction) -> crate::Result<&Self> {
        TEST.append(&mut self.set_from_transaction_nodes(trans)?);
        Ok(&self)
    }

    pub fn set_from_transaction_nodes(&self, trans: &Transaction) -> crate::Result<Vec<Node<PAT>>> {
        self.verify_size(trans)?;
        let bit_actions = trans.to_symbols()?;
        let mut nodes: Vec<Node<PAT>> = self.update_from_bit_actions(&bit_actions)?;
        if let Some(ovl) = trans.overlay.as_ref() {
            let mut o = ovl.clone();
            o.pin_ids = Some(self.as_ids());
            nodes.insert(0, node!(PAT::Overlay, o, None));
        }
        Ok(nodes)
    }

    pub fn get_actions(&self) -> Vec<PinAction> {
        self.pins
            .iter()
            .map(|p| p.action.read().unwrap().clone())
            .collect()
    }

    pub fn get_reset_actions(&self) -> Vec<PinAction> {
        self.pins
            .iter()
            .map(|p| {
                if let Some(ra) = &p.reset_action {
                    ra.clone()
                } else {
                    PinAction::highz()
                }
            })
            .collect()
    }

    pub fn reset(&self) -> &Self {
        self.pins.iter().for_each(|p| {
            let a;
            if let Some(ra) = &p.reset_action {
                a = ra.clone();
            } else {
                a = PinAction::highz();
            }
            let mut paction = p.action.write().unwrap();
            *paction = a.clone();
            TEST.push(node!(PAT::PinAction, p.id, a.to_string(), None));
        });
        &self
    }

    fn verify_size(&self, trans: &Transaction) -> crate::Result<()> {
        if trans.width != self.pins.len() {
            Err(error!(
                "Error in PinCollection: Transaction of width {} does not match PinCollection size {}. PC: {:?}, Transaction: {:?}",
                trans.width,
                self.pins.len(),
                self,
                trans
            ))
        } else {
            Ok(())
        }
    }

    /// Internal function to update all pins in this collection from a single PinAction
    fn update_from_bit_actions(
        &self,
        bit_actions: &Vec<(String, bool, bool)>,
    ) -> crate::Result<Vec<Node<PAT>>> {
        let mut action_nodes: Vec<Node<PAT>> = vec![];

        let mut this_grp_nodes: Vec<Node<PAT>> = vec![];
        let mut this_grp_action: Vec<String> = vec![];
        let mut current_cnt = 0;
        let mut grp_idx = 0;

        for (i, bit_action) in bit_actions.iter().enumerate() {
            if self.grp_ids.is_some() {
                let p = &self.pins[i];

                let n = node!(PAT::PinAction, p.id, bit_action.0.to_string(), None);
                let context_node: Option<Node<PAT>> = None;
                // if bit_action.1 {
                //     // context_node = Some(node!(PAT::Overlay, overlay_str.clone(), Some(p.id), Some(bit_action.0.to_string()), None));
                // }
                // if bit_action.2 {
                //     // let capture_node = node!(PAT::Capture, Some(p.id), Some(bit_action.0.to_string()), None);
                //     // if let Some(mut cnode) = context_node.as_mut() {
                //     //     cnode.add_child(capture_node);
                //     // } else {
                //     //     context_node = Some(capture_node);
                //     // }
                // }
                if let Some(mut cnode) = context_node {
                    cnode.add_child(n);
                    this_grp_nodes.push(cnode);
                } else {
                    this_grp_nodes.push(n);
                }

                //this_grp_nodes.push(node!(PAT::PinAction, p.id, bit_action.to_string(), None));
                this_grp_action.push(bit_action.0.to_string());
                let mut paction = p.action.write().unwrap();
                *paction = PinAction::new(bit_action.0.to_string());
                current_cnt += 1;
                if current_cnt == self.grp_ids.as_ref().unwrap()[grp_idx].1 {
                    let mut n = node!(
                        PAT::PinGroupAction,
                        self.grp_ids.as_ref().unwrap()[grp_idx].0,
                        this_grp_action,
                        None
                    );
                    n.add_children(this_grp_nodes);
                    action_nodes.push(n);
                    this_grp_nodes = vec![];
                    grp_idx += 1;
                    current_cnt = 0;
                    this_grp_action = vec![];
                }
            } else {
                // no pin groups. Just push the straight pins
                let p = &self.pins[i];
                let mut paction = p.action.write().unwrap();
                *paction = PinAction::new(bit_action.0.to_string());

                let context_node: Option<Node<PAT>> = None;
                let n = node!(PAT::PinAction, p.id, bit_action.0.to_string(), None);
                // if bit_action.1 {
                //     // context_node = Some(node!(PAT::Overlay, overlay_str.clone(), Some(p.id), Some(bit_action.0.to_string()), None));
                // }
                // if bit_action.2 {
                //     // let capture_node = node!(PAT::Capture, Some(p.id), Some(bit_action.0.to_string()), None);
                //     // if let Some(mut cnode) = context_node.as_mut() {
                //     //     cnode.add_child(capture_node);
                //     // } else {
                //     //     context_node = Some(capture_node);
                //     // }
                // }
                if let Some(mut cnode) = context_node {
                    cnode.add_child(n);
                    action_nodes.push(cnode);
                } else {
                    action_nodes.push(n);
                }
                //action_nodes.push(node!(PAT::PinAction, p.id, bit_action.to_string(), None));
            }
        }
        Ok(action_nodes)
    }

    /// Generates a transaction on the pin bus and pushes the nodes to the AST
    pub fn push_transaction(&self, trans: &Transaction) -> crate::Result<&Self> {
        TEST.append(&mut self.push_transaction_nodes(trans)?);
        Ok(&self)
    }

    /// Generate a transaction on the pin bus. The data, data width, operation, and overlay settings should
    /// all be encapsulated in the transaction struct
    pub fn push_transaction_nodes(&self, trans: &Transaction) -> crate::Result<Vec<Node<PAT>>> {
        let bit_actions = trans.to_symbols()?;
        let mut pin_states: Vec<Node<PAT>> = vec![];
        if let Some(c) = &trans.capture {
            let capture_sym;
            // Push the capture node and note if a custom character is given.
            // Masking will be resolved in trans.bit_actions function
            if let Some(s) = &c.symbol {
                capture_sym = Some(s.clone());
            } else {
                capture_sym = None;
            }
            pin_states.push(node!(
                PAT::Capture,
                crate::Capture {
                    pin_ids: Some(self.as_ids()),
                    cycles: Some(self.cycles_to_push(trans)),
                    enables: c.enables.clone(),
                    symbol: capture_sym.clone()
                },
                None
            ));
        }

        if let Some(o) = &trans.overlay {
            let mut ovl = o.clone();
            if ovl.cycles.is_none() {
                ovl.cycles = Some(self.cycles_to_push(trans));
            }
            pin_states.push(node!(PAT::Overlay, ovl, None));
        }

        for (_idx, chunk) in bit_actions.chunks(self.pins.len()).enumerate() {
            let mut this_cycle: Vec<Node<PAT>> = vec![];
            let mut this_grp_nodes: Vec<Node<PAT>> = vec![];
            let mut this_grp_action: Vec<String> = vec![];
            let mut current_cnt = 0;
            let mut grp_idx = 0;

            for (pos, bit_action) in chunk.iter().enumerate() {
                if self.grp_ids.is_some() {
                    let p = &self.pins[pos];
                    let n = node!(PAT::PinAction, p.id, bit_action.0.to_string(), None);
                    let context_node: Option<Node<PAT>> = None;
                    // if bit_action.1 {
                    //     // context_node = Some(node!(PAT::Overlay, trans.overlay_string.clone(), Some(p.id), Some(bit_action.0.to_string()), None));
                    // }
                    // if bit_action.2 {
                    //     // let capture_node = node!(PAT::Capture, Some(p.id), Some(bit_action.0.to_string()), None);
                    //     // if let Some(mut cnode) = context_node.as_mut() {
                    //     //     cnode.add_child(capture_node);
                    //     // } else {
                    //     //     context_node = Some(capture_node);
                    //     // }
                    // }
                    if let Some(mut cnode) = context_node {
                        cnode.add_child(n);
                        this_grp_nodes.push(cnode);
                    } else {
                        this_grp_nodes.push(n);
                    }
                    this_grp_action.push(bit_action.0.to_string());
                    let mut paction = p.action.write().unwrap();
                    *paction = PinAction::new(bit_action.0.to_string());
                    current_cnt += 1;
                    if current_cnt == self.grp_ids.as_ref().unwrap()[grp_idx].1 {
                        let mut n = node!(
                            PAT::PinGroupAction,
                            self.grp_ids.as_ref().unwrap()[grp_idx].0,
                            this_grp_action,
                            None
                        );
                        n.add_children(this_grp_nodes);
                        this_cycle.push(n);
                        this_grp_nodes = vec![];
                        grp_idx += 1;
                        current_cnt = 0;
                        this_grp_action = vec![];
                    }
                } else {
                    // no pin groups. Just push the straight pins
                    let p = &self.pins[pos];

                    let mut context_node: Option<Node<PAT>> = None;
                    let n = node!(PAT::PinAction, p.id, bit_action.0.to_string(), None);
                    if bit_action.1 {
                        // context_node = Some(node!(PAT::Overlay, trans.overlay_string.clone(), Some(p.id), Some(bit_action.0.to_string()), None));
                    }
                    if bit_action.2 {
                        // let capture_node = node!(PAT::Capture, Some(p.id), Some(bit_action.0.to_string()), None);
                        // if let Some(mut cnode) = context_node.as_mut() {
                        //     cnode.add_child(capture_node);
                        // } else {
                        //     context_node = Some(capture_node);
                        // }
                    }
                    if context_node.is_some() {
                        context_node.as_mut().unwrap().add_child(n);
                        this_cycle.push(context_node.unwrap());
                    } else {
                        this_cycle.push(n);
                    }

                    let mut paction = p.action.write().unwrap();
                    *paction = PinAction::new(bit_action.0.to_string());
                }
            }
            // Push the cycle updates and cycle the tester
            pin_states.append(&mut this_cycle);
            pin_states.push(repeat2_node(1, !trans.has_overlay()));
        }
        Ok(pin_states)
    }

    /// Push a cycle to the AST
    pub fn cycle(&self) -> &Self {
        cycle();
        &self
    }

    pub fn cycles(&self, count: usize) -> &Self {
        self.repeat(count as u32)
    }

    /// Add number of compressed cycles indicated by count
    pub fn repeat(&self, count: u32) -> &Self {
        repeat(count);
        &self
    }

    /// Repeat with two arguments - count and compressable
    pub fn repeat2(&self, count: u32, compressable: bool) -> &Self {
        repeat2(count, compressable);
        &self
    }

    pub fn cycles_to_push(&self, trans: &Transaction) -> usize {
        num::integer::Integer::div_ceil(&trans.width, &self.width())
    }

    // /// Find the most recent nodes in the AST which set the current pin action and update the internal pin state accordingly
    // pub fn update_actions(&self, dut: &crate::Dut) -> crate::Result<()> {
    //     let mut pins_to_update = self.pin_ids.clone();
    //     let mut cnt = 0;
    //     while pins_to_update.len() > 0 {
    //         match TEST.get_with_descendants(cnt)?.attrs {
    //             Attrs::PinAction(pin_id, symbol, _metadata) => {
    //                 let pos = pins_to_update.iter().position( |i| *i == pin_id);
    //                 if let Some(p) = pos {
    //                     pins_to_update.remove(p);
    //                     *dut.pins[pin_id].action.write().unwrap() = PinActions::from_delimiter_optional(&symbol)?;
    //                 }
    //             }
    //             _ => {}
    //         }
    //         cnt += 1;
    //     }
    //     Ok(())
    // }
}

#[derive(Debug, Copy, Clone)]
pub enum Endianness {
    LittleEndian,
    BigEndian,
}

impl Model {
    pub fn register_pin(
        &mut self,
        pin_group_id: usize,
        physical_pin_id: usize,
        name: &str,
        reset_action: Option<PinAction>,
        endianness: Option<Endianness>,
    ) -> Result<(PinGroup, Pin)> {
        let pin_group = PinGroup::new(
            self.id,
            pin_group_id,
            name,
            vec![physical_pin_id],
            endianness,
        );
        let physical_pin = Pin::new(self.id, physical_pin_id, name.to_string(), reset_action);
        self.pin_groups.insert(name.to_string(), pin_group_id);
        self.pins.insert(name.to_string(), physical_pin_id);
        Ok((pin_group, physical_pin))
    }

    pub fn register_pin_group(
        &mut self,
        pin_group_id: usize,
        name: &str,
        pin_ids: Vec<usize>,
        endianness: Option<Endianness>,
    ) -> Result<PinGroup> {
        let pin_group = PinGroup::new(
            self.id,
            pin_group_id,
            &name.to_string(),
            pin_ids,
            endianness,
        );
        self.pin_groups.insert(name.to_string(), pin_group_id);
        Ok(pin_group)
    }

    pub fn get_pin_id(&self, name: &str) -> Option<usize> {
        match self.pins.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }

    pub fn get_pin_group_id(&self, name: &str) -> Option<usize> {
        match self.pin_groups.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }
}

impl Dut {
    pub fn add_pin(
        &mut self,
        model_id: usize,
        name: &str,
        width: Option<u32>,
        offset: Option<u32>,
        reset_action: Option<Vec<PinAction>>,
        endianness: Option<Endianness>,
    ) -> Result<&PinGroup> {
        // Check some of the parameters before we go much further. We can error out quickly if something is awry.
        // Check the width and offset
        if !width.is_some() && offset.is_some() {
            bail!(
                "Can not add pin {} with a given offset but no width option!",
                name
            );
        } else if self.get_pin_group(model_id, name).is_some() {
            bail!(
                "Pin '{}' already exists on model '{}'!",
                name,
                self.models[model_id].name
            );
        }

        // Check that the given reset pin actions fit within the width of the pins to add and that they
        // are valid pin action characters.
        if let Some(ref r) = reset_action {
            if r.len() != (width.unwrap_or(1) as usize) {
                bail!(
                    "PinActions of length {} must match width {}!",
                    r.len(),
                    width.unwrap_or(1)
                );
            }
        }

        // Resolve the names first - if there's a problem with one of the names, an error will generated here but passed up
        // to the frontend, which should end the program. However, the user could catch the exception, which would leave the
        // backend here in half-complete state.
        // Just to be safe, resolve and check the names first before adding anything.
        let mut names: Vec<String> = vec![];
        if let Some(w) = width {
            if w < 1 {
                bail!("Width cannot be less than 1! Received {}", w);
            }
            let o = offset.unwrap_or(0);
            for i in o..(o + w) {
                let n = format!("{}{}", name, i).to_string();
                if self.get_pin_group(model_id, name).is_some() {
                    bail!(
                        "Can not add pin {}, derived by adding pin {} of width {} with offset {}, because it conflicts with a current pin or alias name!",
                        n,
                        name,
                        w,
                        o,
                    );
                }
                names.push(n);
            }
        } else {
            names.push(name.to_string());
        }

        // Checks passed, so add the pins.
        let (mut pin_group_id, mut physical_pin_id);
        {
            pin_group_id = self.pin_groups.len();
            physical_pin_id = self.pins.len();
        }
        {
            let model = &mut self.models[model_id];
            let mut ra = None;
            for (i, n) in names.iter().enumerate() {
                if let Some(ref r) = reset_action {
                    ra = Some(r[i].clone());
                }
                let (pin_group, mut physical_pin) = model.register_pin(
                    pin_group_id,
                    physical_pin_id,
                    &n,
                    ra.clone(),
                    endianness,
                )?;
                if names.len() > 1 {
                    physical_pin.groups.insert(name.to_string(), i);
                }
                self.pin_groups.push(pin_group);
                self.pins.push(physical_pin);
                pin_group_id += 1;
                physical_pin_id += 1;
            }
        }
        if offset.is_some() || width.is_some() {
            // Add the group containing all the pins we just added, with index/offset.
            // But, the actual requested group hasn't been added yet.
            // If the offset and width are None, then group has the provided name.
            self.group_pins_by_name(model_id, name, names, endianness)?;
            Ok(&self.pin_groups[pin_group_id])
        } else {
            Ok(&self.pin_groups[pin_group_id - 1])
        }
    }

    pub fn add_pin_alias(&mut self, model_id: usize, name: &str, alias: &str) -> Result<()> {
        // First, check that the pin exists.
        if self.models[model_id].pin_groups.contains_key(alias) {
            bail!(
                "Could not alias '{}' to '{}', as '{}' already exists!",
                name,
                alias,
                alias
            );
        }

        let (grp, id, ids);
        if let Some(idx) = self.models[model_id].pin_groups.get(name) {
            id = self.pin_groups.len();
            let p = &self.pin_groups[*idx];
            grp = PinGroup::new(
                model_id,
                id,
                alias,
                p.pin_ids.clone(),
                Option::Some(p.endianness),
            );
            ids = p.pin_ids.clone();
        } else {
            bail!(
                "Could not alias '{}' to '{}', as '{}' doesn't exists!",
                name,
                alias,
                name
            );
        }
        for pid in ids {
            let p = &mut self.pins[pid];
            p.aliases.push(alias.to_string())
        }
        self.models[model_id]
            .pin_groups
            .insert(alias.to_string(), id);
        self.pin_groups.push(grp);
        Ok(())
    }

    pub fn group_pins_by_name(
        &mut self,
        model_id: usize,
        name: &str,
        pins: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<&PinGroup> {
        let id;
        {
            id = self.pin_groups.len();
        }
        let pnames = self.verify_names(model_id, &pins)?;
        for (i, pname) in pnames.iter().enumerate() {
            let p = self._get_mut_pin(model_id, pname)?;
            p.groups.insert(String::from(name), i);
        }

        let ids = pnames
            .iter()
            .map(|p| self._get_pin(model_id, p).unwrap().id)
            .collect();
        let model = &mut self.models[model_id];
        self.pin_groups
            .push(model.register_pin_group(id, name, ids, endianness)?);
        Ok(&self.pin_groups[id])
    }

    /// Given a group/collection of pin names, verify:
    ///     * Each pin exist
    ///     * Each pin is unique (no duplicate pins) AND it points to a unique physical pin. That is, each pin is unique after resolving aliases.
    /// If all the above is met, we can group/collect these names.
    pub fn verify_names(&self, model_id: usize, names: &Vec<String>) -> Result<Vec<String>> {
        let mut physical_names: Vec<String> = vec![];
        for (_i, pin_name) in names.iter().enumerate() {
            if pin_name.starts_with("/") && pin_name.ends_with("/") {
                let mut regex_str = pin_name.clone();
                regex_str.pop();
                regex_str.remove(0);
                let regex = Regex::new(&regex_str).unwrap();

                let mut _pin_names: Vec<String> = vec![];
                for (name_str, grp_id) in self.models[model_id].pin_groups.iter() {
                    if regex.is_match(name_str) {
                        let grp = &self.pin_groups[*grp_id];
                        let n = grp
                            .pin_ids
                            .iter()
                            .map(|pid| self.pins[*pid].name.to_string())
                            .collect::<Vec<String>>();
                        for _name_str in n.iter() {
                            // for _name_str in grp.pin_names.iter() {
                            if physical_names.contains(_name_str) {
                                bail!("Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", name_str, regex_str, _name_str);
                            }
                        }
                        // _pin_names.extend(grp.pin_names.clone())
                        _pin_names.extend(n);
                    }
                }
                _pin_names.sort();
                physical_names.extend(_pin_names);
            } else if let Some(p) = self.resolve_to_physical_pin(model_id, pin_name) {
                if physical_names.contains(&p.name) {
                    bail!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", pin_name, p.name);
                }
                if let Some(p) = self.get_pin_group(model_id, pin_name) {
                    let n = p
                        .pin_ids
                        .iter()
                        .map(|pid| self.pins[*pid].name.clone())
                        .collect::<Vec<String>>();
                    // physical_names.extend_from_slice(&p.pin_names);
                    physical_names.extend_from_slice(&n);
                }
            } else {
                bail!(
                    "Can not collect pin '{}' because it does not exist!",
                    pin_name
                );
            }
        }
        Ok(physical_names.clone())
    }

    pub fn collect_grp_ids_as_pin_ids(
        &self,
        grps: &Vec<(String, usize)>,
    ) -> crate::Result<Vec<usize>> {
        let mut physical_ids: Vec<usize> = vec![];
        for (_i, identifier) in grps.iter().enumerate() {
            if identifier.0.starts_with("/") && identifier.0.ends_with("/") {
                let mut regex_str = identifier.0.clone();
                regex_str.pop();
                regex_str.remove(0);
                let regex = Regex::new(&regex_str).unwrap();

                let mut _pin_names: Vec<usize> = vec![];
                for (name_str, grp_id) in self.models[identifier.1].pin_groups.iter() {
                    if regex.is_match(name_str) {
                        let grp = &self.pin_groups[*grp_id];
                        // for _name_str in grp.pin_names.iter() {
                        //     if physical_ids.contains(&self._get_pin(identifier.1, _name_str)?.id) {
                        //         bail!("Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", name_str, regex_str, _name_str);
                        //     }
                        // }
                        for pid in grp.pin_ids.iter() {
                            if physical_ids.contains(&pid) {
                                bail!(
                                    "Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!",
                                    name_str,
                                    regex_str,
                                    &self.pins[*pid].name
                                );
                            }
                        }
                        // _pin_names.extend(grp.pin_names.iter().map( |n| self._get_pin(identifier.1, n).unwrap().id).collect::<Vec<usize>>());
                        _pin_names.extend(&grp.pin_ids.clone());
                    }
                }
                _pin_names.sort();
                physical_ids.extend(_pin_names);
            } else if let Some(p) = self.resolve_to_physical_pin(identifier.1, &identifier.0) {
                if physical_ids.contains(&p.id) {
                    bail!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", identifier.0, p.name);
                }
                if let Some(p) = self.get_pin_group(identifier.1, &identifier.0) {
                    // physical_ids.extend(&p.pin_names.iter().map( |n| self._get_pin(identifier.1, n).unwrap().id).collect::<Vec<usize>>());
                    physical_ids.extend(&p.pin_ids.clone());
                }
            } else {
                bail!(
                    "Can not collect pin '{}' because it does not exist!",
                    identifier.0
                );
            }
        }
        Ok(physical_ids.clone())
    }

    pub fn collect(
        &self,
        grps: &Vec<(String, usize)>,
        endianness: Option<Endianness>,
    ) -> Result<PinStore> {
        let pids = self.collect_grp_ids_as_pin_ids(grps)?;
        Ok(PinStore::new(pids, endianness))
    }

    pub fn pin_names_contain(
        &self,
        model_id: usize,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<bool> {
        let result = self.find_in_names(model_id, names, query_name)?.is_some();
        Ok(result)
    }

    pub fn find_in_names(
        &self,
        model_id: usize,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<Option<usize>> {
        if let Some(p) = self.get_pin(model_id, query_name) {
            let idx = names
                .iter()
                .position(|name| p.name == *name || p.aliases.contains(name));
            if let Some(_idx) = idx {
                Ok(Option::Some(_idx))
            } else {
                // Group name wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query name doesn't exists. Raise an error.
            Err(error!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            ))
        }
    }

    /// Given a pin or alias name, finds either its name or alias in the group.
    pub fn index_of(&self, model_id: usize, name: &str, query_name: &str) -> Result<Option<usize>> {
        if !self.models[model_id].pin_groups.contains_key(name) {
            // Pin group doesn't exists. Raise an error.
            bail!(
                "Group {} does not exists! Cannot lookup index for {} in this group!",
                name,
                query_name
            );
        }

        if let Some(p) = self.get_pin(model_id, query_name) {
            if let Some(idx) = p.groups.get(name) {
                Ok(Option::Some(*idx))
            } else {
                // Group name wasn't found in this pin's groups.
                // Pin doesn't belong to that group.
                Ok(Option::None)
            }
        } else {
            // The query name doesn't exists. Raise an error.
            Err(error!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            ))
        }
    }

    pub fn resolve_to_physical_pin(&self, model_id: usize, name: &str) -> Option<&Pin> {
        if let Some(grp) = self.get_pin_group(model_id, name) {
            return Some(&self.pins[grp.pin_ids[0]]);
        }
        Option::None
    }

    pub fn data_fits_in_pins(&mut self, pins: &Vec<String>, data: u32) -> Result<()> {
        let two: u32 = 2;
        if data > (two.pow(pins.len() as u32) - 1) {
            Err(error!(
                "Data {} does not fit in Pin collection of size {} - Cannot set data!",
                data,
                pins.len()
            ))
        } else {
            Ok(())
        }
    }

    pub fn verify_data_fits(&mut self, width: u32, data: u32) -> Result<()> {
        let two: u32 = 2;
        if data > (two.pow(width) - 1) {
            Err(error!(
                "Data {} does not fit in pins with width of {}!",
                data, width
            ))
        } else {
            Ok(())
        }
    }

    pub fn verify_action_string_fits(&self, width: u32, action_string: &Vec<u8>) -> Result<()> {
        if action_string.len() != (width as usize) {
            Err(error!(
                "Action string of length {} must match width {}!",
                action_string.len(),
                width
            ))
        } else {
            Ok(())
        }
    }

    /// Given a pin name, check if the pin or any of its aliases are present in pin group.
    pub fn pin_group_contains(
        &self,
        model_id: usize,
        name: &str,
        query_name: &str,
    ) -> Result<bool> {
        let result = self.index_of(model_id, name, query_name)?.is_some();
        Ok(result)
    }

    pub fn contains(&self, model_id: usize, name: &str) -> bool {
        return self.get_pin_group(model_id, name).is_some();
    }

    pub fn _contains(&self, model_id: usize, name: &str) -> bool {
        return self.get_pin(model_id, name).is_some();
    }

    pub fn _resolve_group_to_physical_pins(
        &self,
        model_id: usize,
        name: &str,
    ) -> Result<Vec<&Pin>> {
        let mut retn: Vec<&Pin> = vec![];
        let grp = self._get_pin_group(model_id, name)?;
        for p_id in grp.pin_ids.iter() {
            retn.push(&self.pins[*p_id]);
        }
        Ok(retn)
    }

    pub fn _resolve_groups_to_physical_pin_ids(
        &self,
        pins: &Vec<(usize, String)>,
    ) -> Result<Vec<Vec<usize>>> {
        let mut retn: Vec<Vec<usize>> = vec![];
        for lookup in pins.iter() {
            let ppins = self._resolve_group_to_physical_pins(lookup.0, &lookup.1)?;
            retn.push(ppins.iter().map(|p| p.id).collect::<Vec<usize>>());
        }
        Ok(retn)
    }

    pub fn _resolve_to_flattened_pins(&self, pins: &Vec<(usize, String)>) -> Result<Vec<&Pin>> {
        let mut retn: Vec<&Pin> = vec![];
        for lookup in pins.iter() {
            let mut ppins = self._resolve_group_to_physical_pins(lookup.0, &lookup.1)?;
            retn.append(&mut ppins);
        }
        Ok(retn)
    }

    /// Given a pin group name and model ID, converts it to a tuple containing:
    ///  [0] -> Vec<usize> containing the physical pin IDs of the pins in this group
    ///  [1] -> usize -> the resolved pin group ID
    pub fn pin_group_to_ids(
        &self,
        model_id: usize,
        pin_grp_name: &str,
    ) -> Result<(Vec<usize>, usize)> {
        let p_ids: Vec<usize> = self
            ._resolve_group_to_physical_pins(model_id, pin_grp_name)?
            .iter()
            .map(|p| p.id)
            .collect();
        Ok((p_ids, self._get_pin_group(model_id, pin_grp_name)?.id))
    }
}

#[derive(Debug, Clone)]
pub struct StateTracker {
    pins: IndexMap<String, Vec<String>>,
    _model_id: usize,
}

impl StateTracker {
    /// Creates a new state storage container. Creating a new instance populates the given groups with their reset data and actions.
    pub fn new(model_id: usize, pin_header_id: Option<usize>, dut: &Dut) -> Self {
        let mut pins: IndexMap<String, Vec<String>> = IndexMap::new();
        if let Some(id) = pin_header_id {
            for n in dut.pin_headers[id].pin_names.iter() {
                let mut states: Vec<String> = vec![];
                for p in dut._resolve_group_to_physical_pins(model_id, n).unwrap() {
                    if let Some(r) = p.reset_action.as_ref() {
                        states.push(r.to_string());
                    } else {
                        states.push("Z".to_string());
                    }
                }
                pins.insert(n.clone(), states);
            }
        } else {
            // No pin header was given. Default pins will be all physical pins on the DUT (Dut, being model ID of 0 here, not the DUT container in general)
            for phys in dut.pins.iter() {
                if phys.model_id == 0 {
                    // Note: the phys name is guaranteed to be in the pin groups, as this physical's pins pin group representation
                    pins.insert(phys.name.clone(), {
                        if let Some(r) = phys.reset_action.as_ref() {
                            vec![r.to_string()]
                        } else {
                            vec!["Z".to_string()]
                        }
                    });
                }
            }
        }
        Self {
            pins: pins,
            _model_id: model_id,
        }
    }

    /// Given a physical pin name, action, and data, updates the state appropriately
    pub fn update(&mut self, grp_id: usize, actions: &Vec<String>, dut: &Dut) -> Result<()> {
        for (i, pid) in dut.pin_groups[grp_id].pin_ids.iter().enumerate() {
            let p = &dut.pins[*pid];
            let action = &actions[i];
            // Check for the header pin in the aliases
            if let Some(states) = self.pins.get_mut(&p.name) {
                states[0] = action.to_string();
                continue;
            }

            // Check for the header pin in the groups
            for (grp, offset) in p.groups.iter() {
                if let Some(states) = self.pins.get_mut(grp) {
                    states[*offset] = action.to_string();
                    continue;
                }
            }

            // Check for the header pin in the aliases
            for alias in p.aliases.iter() {
                if let Some(states) = self.pins.get_mut(alias) {
                    states[0] = action.to_string();
                    continue;
                }
            }
            // bail!(
            //     "Could not resolve physical pin {} to any pins in header {}",
            //     // physical_pin,
            //     &p.name,
            //     self.pins
            //         .keys()
            //         .map(|n| n.to_string())
            //         .collect::<Vec<String>>()
            //         .join(", ")
            // );
        }
        Ok(())
    }

    /// Processes the current state into a vector of 'state strings', where each string corresponds to a tester representation of the actions and data.
    /// E.g.: 'porta': [(PinAction::Drive), 1, (PinAction::HighZ, 0)], 'clk': [(PinAction::Drive), 1], 'reset': [(PinAction::Verify), 0]
    ///     => ['1Z', '1', 'L']
    /// If a header was given, the order will be identical to that from the header. If no header was given, the order will be whatever order was when the default
    /// pins were collected.
    pub fn to_symbols(
        &self,
        target: String,
        dut: &Dut,
        t: &super::timesets::timeset::Timeset,
        overrides: Option<&std::collections::HashMap<usize, String>>,
    ) -> Result<Vec<String>> {
        let mut syms: Vec<String> = vec![];
        let final_states;
        let final_states_ref;
        if let Some(o) = overrides.as_ref() {
            final_states = self.apply_overrides(dut, o)?;
            final_states_ref = &final_states;
        } else {
            final_states_ref = &self.pins;
        }
        for (_n, states) in final_states_ref.iter() {
            let mut s: Vec<String> = vec![];
            for action in states.iter() {
                s.push(t._resolve_pin_action(target.clone(), &PinAction::new(action))?);
            }
            syms.push(s.join(""));
        }
        Ok(syms)
    }

    pub fn apply_overrides(
        &self,
        dut: &Dut,
        overrides: &std::collections::HashMap<usize, String>,
    ) -> Result<IndexMap<String, Vec<String>>> {
        let mut state_overrides = self.pins.clone();
        for (ppid, override_state) in overrides {
            let p = &dut.pins[*ppid];
            // Check for the header pin in the aliases
            if let Some(states) = state_overrides.get_mut(&p.name) {
                states[0] = override_state.to_string();
                continue;
            }

            // Check for the header pin in the groups
            for (grp, offset) in p.groups.iter() {
                if let Some(states) = state_overrides.get_mut(grp) {
                    states[*offset] = override_state.to_string();
                    continue;
                }
            }

            // Check for the header pin in the aliases
            for alias in p.aliases.iter() {
                if let Some(states) = state_overrides.get_mut(alias) {
                    states[0] = override_state.to_string();
                    continue;
                }
            }
        }
        Ok(state_overrides)
    }

    pub fn names(&self) -> Vec<String> {
        self.pins
            .keys()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
    }

    pub fn contains_action(&self, action: PinAction) -> bool {
        for (_pin, actions) in self.pins.iter() {
            if actions.iter().any(|a| a.to_string() == action.to_string()) {
                return true;
            }
        }
        false
    }
}
