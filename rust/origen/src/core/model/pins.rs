pub mod pin;
pub mod pin_collection;
pub mod pin_group;
pub mod pin_header;
use super::super::dut::Dut;
use crate::error::Error;
use crate::generator::ast::{Node};
use crate::{Transaction, node, TEST};
use crate::testers::vector_based::api::{cycle, repeat, repeat2, repeat2_node};
use crate::standards::actions::*;
use crate::generator::ast::Attrs;

use regex::Regex;

use super::Model;
use indexmap::IndexMap;
use pin::{Pin, PinActions, ResolvePinActions};
use pin_collection::PinCollection;
use pin_group::PinGroup;

#[derive(Debug, Clone)]
pub struct PinBus {
    grp_ids: Option<Vec<(usize, usize)>>,
    pin_ids: Vec<usize>,
}

impl PinBus {
    pub fn from_group(dut: &crate::Dut, grp_name: &str, model_id: usize) -> crate::Result<Self> {
        let p_ids: Vec<usize> = dut._resolve_group_to_physical_pins(
            model_id,
            grp_name
        )?.iter().map( |p| p.id).collect();
        Ok(Self {
            grp_ids: Some(vec!((dut._get_pin_group(model_id, grp_name)?.id, p_ids.len()))),
            pin_ids: p_ids
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
    //         return Err(Error::new(&format!(
    //             "Duplicate physical pins detected when creating PinBus from {:?} - (resolved pin IDs: {:?}, unique pin IDs {:?})",
    //             grps,
    //             p_ids,
    //             temp
    //         )))
    //     }
    //     Ok(Self {
    //         grp_ids: Some(grp_ids),
    //         pin_ids: p_ids
    //     })
    // }

    /// Applies the drive-high symbol to all the pins on this bus and pushes them to the AST
    pub fn drive_high(&self) -> &Self {
        TEST.append(&mut self.drive_high_nodes());
        &self
    }

    /// Applies the drive-high symbol to all the pins on this bus and returns the nodes without pushing them to the AST
    pub fn drive_high_nodes(&self) -> Vec<Node> {
        self.drive_nodes(true)
    }

    /// Identical to "drive_high" except uses the drive-low symbol instead
    pub fn drive_low(&self) -> &Self {
        TEST.append(&mut self.drive_low_nodes());
        &self
    }

    /// Identical to "drive_low_nodes" except uses the drive-low symbol instead
    pub fn drive_low_nodes(&self) -> Vec<Node> {
        self.drive_nodes(false)
    }

    /// Drives all pins to either the drive-high character, if 'state' is true,
    /// or the drive-low character, if the 'state is false
    pub fn drive(&self, state: bool) -> &Self {
        TEST.append(&mut self.drive_nodes(state));
        &self
    }

    pub fn drive_nodes(&self, state: bool) -> Vec<Node> {
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

    pub fn verify_high_nodes(&self) -> Vec<Node> {
        self.verify_nodes(true)
    }

    pub fn verify_low(&self) -> &Self {
        TEST.append(&mut self.verify_low_nodes());
        &self
    }

    pub fn verify_low_nodes(&self) -> Vec<Node> {
        self.verify_nodes(false)
    }

    pub fn verify(&self, state: bool) -> &Self {
        TEST.append(&mut self.verify_nodes(state));
        &self
    }

    pub fn verify_nodes(&self, state: bool) -> Vec<Node> {
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

    pub fn capture_nodes(&self) -> Vec<Node> {
        self.set_action_nodes(CAPTURE)
    }

    pub fn highz(&self) -> &Self {
        TEST.append(&mut self.highz_nodes());
        &self
    }

    pub fn highz_nodes(&self) -> Vec<Node> {
        self.set_action_nodes(HIGHZ)
    }

    /// Sets all the pins in this bus to an arbitrary action, pushing the nodes onto the AST
    pub fn set_action(&self, action: &str) -> &Self {
        TEST.append(&mut self.set_action_nodes(action));
        &self
    }

    /// Sets all the pins in this bus to an arbitrary action, returning the nodes without pushing to the AST
    pub fn set_action_nodes(&self, action: &str) -> Vec<Node> {
        if let Some(grps) = &self.grp_ids {
            let mut retn = vec![];
            let mut pin_ids_offset = 0;
            for (_i, grp) in grps.iter().enumerate() {
                let mut grp_node = node!(PinGroupAction, grp.0, vec![action.to_string(); self.pin_ids.len()], None);
                grp_node.add_children((0..grp.1).map( |pin_i| node!(PinAction, self.pin_ids[pin_ids_offset + pin_i], action.to_string(), None)).collect());
                retn.push(grp_node);
                pin_ids_offset += grp.1;
            }
            retn
        } else {
            self.pin_ids.iter().map( |id| node!(PinAction, *id, action.to_string(), None)).collect()
        }
    }

    /// Generates a transaction on the pin bus and pushes the nodes to the AST
    pub fn push_transaction(&self, trans: &Transaction) -> crate::Result<&Self> {
        TEST.append(&mut self.push_transaction_nodes(trans)?);
        Ok(&self)
    }

    /// Generate a transaction on the pin bus. The data, data width, operation, and overlay settings should
    /// all be encapsulated in the transaction struct
    pub fn push_transaction_nodes(&self, trans: &Transaction) -> crate::Result<Vec<Node>> {
        let bit_actions = trans.to_symbols()?;
        let mut pin_states: Vec<Node> = vec!();

        for (_idx, chunk) in bit_actions.chunks(self.pin_ids.len()).enumerate() {
            let mut this_cycle: Vec<Node> = vec![];
            let mut this_grp_nodes: Vec<Node> = vec![];
            let mut this_grp_action: Vec<String> = vec![];
            let mut current_cnt = 0;
            let mut grp_idx = 0;

            for (pos, bit_action) in chunk.iter().enumerate() {
                if self.grp_ids.is_some() {
                    this_grp_nodes.push(node!(PinAction, self.pin_ids[pos], bit_action.to_string(), None));
                    this_grp_action.push(bit_action.to_string());
                    current_cnt += 1;
                    if current_cnt == self.grp_ids.as_ref().unwrap()[grp_idx].1 {
                        let mut n = node!(PinGroupAction, self.grp_ids.as_ref().unwrap()[grp_idx].0, this_grp_action, None);
                        n.add_children(this_grp_nodes);
                        this_cycle.push(n);
                        this_grp_nodes = vec![];
                        grp_idx += 1;
                        current_cnt = 0;
                        this_grp_action = vec![];
                    }
                } else {
                    // no pin groups. Just push the straight pins
                    this_cycle.push(node!(PinAction, self.pin_ids[pos], bit_action.to_string(), None));
                }
            }
            // Push the cycle updates and cycle the tester
            pin_states.append(&mut this_cycle);
            pin_states.push(repeat2_node(1, !trans.overlay_string.is_some()));
        }
        Ok(pin_states)
    }

    /// Push a cycle to the AST
    pub fn cycle(&self) -> &Self {
        cycle();
        &self
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

    /// Find the most recent nodes in the AST which set the current pin action and update the internal pin state accordingly
    pub fn update_actions(&self, dut: &crate::Dut) -> crate::Result<()> {
        let mut pins_to_update = self.pin_ids.clone();
        let mut cnt = 0;
        while pins_to_update.len() > 0 {
            match TEST.get_with_descendants(cnt)?.attrs {
                Attrs::PinAction(pin_id, symbol, _metadata) => {
                    let pos = pins_to_update.iter().position( |i| *i == pin_id);
                    if let Some(p) = pos {
                        pins_to_update.remove(p);
                        *dut.pins[pin_id].action.write().unwrap() = PinActions::from_delimiter_optional(&symbol)?;
                    }
                }
                _ => {}
            }
            cnt += 1;
        }
        Ok(())
    }
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
        reset_data: Option<u32>,
        reset_action: Option<PinActions>,
        endianness: Option<Endianness>,
    ) -> Result<(PinGroup, Pin), Error> {
        let pin_group = PinGroup::new(
            self.id,
            pin_group_id,
            name.to_string(),
            vec![name.to_string()],
            endianness,
        );
        let physical_pin = Pin::new(
            self.id,
            physical_pin_id,
            name.to_string(),
            reset_data,
            reset_action,
        );
        self.pin_groups.insert(name.to_string(), pin_group_id);
        self.pins.insert(name.to_string(), physical_pin_id);
        Ok((pin_group, physical_pin))
    }

    pub fn register_pin_group(
        &mut self,
        pin_group_id: usize,
        name: &str,
        pins: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<PinGroup, Error> {
        let pin_group = PinGroup::new(self.id, pin_group_id, name.to_string(), pins, endianness);
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
        reset_data: Option<u32>,
        reset_action: Option<Vec<PinActions>>,
        endianness: Option<Endianness>,
    ) -> Result<&PinGroup, Error> {
        // Check some of the parameters before we go much further. We can error out quickly if something is awry.
        // Check the width and offset
        if !width.is_some() && offset.is_some() {
            return Err(Error::new(&format!(
                "Can not add pin {} with a given offset but no width option!",
                name
            )));
        } else if self.get_pin_group(model_id, name).is_some() {
            return Err(Error::new(&format!(
                "Pin '{}' already exists on model '{}'!",
                name, self.models[model_id].name
            )));
        }
        let mut rdata = None;

        // Check that the given reset data fits within the width of the pins to add.
        if let Some(r) = reset_data {
            self.verify_data_fits(width.unwrap_or(1), r)?;
            rdata = Some(r);
        }

        // Check that the given reset pin actions fit within the width of the pins to add and that they
        // are valid pin action characters.
        if let Some(ref r) = reset_action {
            if r.len() != (width.unwrap_or(1) as usize) {
                return Err(Error::new(&format!(
                    "PinActions of length {} must match width {}!",
                    r.len(),
                    width.unwrap_or(1)
                )))
            }
        }

        // Resolve the names first - if there's a problem with one of the names, an error will generated here but passed up
        // to the frontend, which should end the program. However, the user could catch the exception, which would leave the
        // backend here in half-complete state.
        // Just to be safe, resolve and check the names first before adding anything.
        let mut names: Vec<String> = vec![];
        if let Some(w) = width {
            if w < 1 {
                return Err(Error::new(&format!(
                    "Width cannot be less than 1! Received {}",
                    w
                )));
            }
            let o = offset.unwrap_or(0);
            for i in o..(o + w) {
                let n = format!("{}{}", name, i).to_string();
                if self.get_pin_group(model_id, name).is_some() {
                    return Err(Error::new(&format!(
                        "Can not add pin {}, derived by adding pin {} of width {} with offset {}, because it conflicts with a current pin or alias name!",
                        n,
                        name,
                        w,
                        o,
                    )));
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
            let (mut rd, mut ra) = (None, None);
            for (i, n) in names.iter().enumerate() {
                if let Some(r) = rdata {
                    rd = Some(r & 0x1);
                    rdata = Some(r >> 1);
                }
                if let Some(ref r) = reset_action {
                    ra = Some(r[i].clone());
                    if let Some(ref _rd) = rd {
                        match ra.as_ref().unwrap() {
                            PinActions::DriveHigh | PinActions::VerifyHigh => {
                                if *_rd == 0 {
                                    return Err(Error::new(&format!(
                                        "Given reset action at position {} conflicts with given reset data",
                                        i,
                                    )));
                                }
                            },
                            PinActions::DriveLow | PinActions::VerifyLow => {
                                if *_rd == 1 {
                                    return Err(Error::new(&format!(
                                        "Given reset action at position {} conflicts with given reset data",
                                        i,
                                    )));
                                }
                            },
                            _ => {}
                        }
                    } else {
                        match ra.as_ref().unwrap() {
                            PinActions::DriveHigh | PinActions::VerifyHigh => rd = Some(1),
                            PinActions::DriveLow | PinActions::VerifyLow => rd = Some(0),
                            _ => {}
                        }
                    }
                }
                let (pin_group, mut physical_pin) =
                    model.register_pin(pin_group_id, physical_pin_id, &n, rd, ra.clone(), endianness)?;
                if names.len() > 1 {
                    physical_pin.groups.insert(name.to_string(), i);
                }
                self.pin_groups.push(pin_group);
                self.pins.push(physical_pin);
                pin_group_id += 1;
                physical_pin_id += 1;
                rd = Option::None;
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

    pub fn add_pin_alias(&mut self, model_id: usize, name: &str, alias: &str) -> Result<(), Error> {
        // First, check that the pin exists.
        if self.models[model_id].pin_groups.contains_key(alias) {
            return Err(Error::new(&format!(
                "Could not alias '{}' to '{}', as '{}' already exists!",
                name, alias, alias
            )));
        }

        let (grp, names, id);
        if let Some(idx) = self.models[model_id].pin_groups.get(name) {
            id = self.pin_groups.len();
            let p = &self.pin_groups[*idx];
            grp = PinGroup::new(
                model_id,
                id,
                String::from(alias),
                p.pin_names.clone(),
                Option::Some(p.endianness),
            );
            names = p.pin_names.clone();
        } else {
            return Err(Error::new(&format!(
                "Could not alias '{}' to '{}', as '{}' doesn't exists!",
                name, alias, name
            )));
        }
        for n in names.iter() {
            let pin = self._get_mut_pin(model_id, n)?;
            pin.aliases.push(String::from(alias));
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
    ) -> Result<&PinGroup, Error> {
        let id;
        {
            id = self.pin_groups.len();
        }
        let pnames = self.verify_names(model_id, &pins)?;
        for (i, pname) in pnames.iter().enumerate() {
            let p = self._get_mut_pin(model_id, pname)?;
            p.groups.insert(String::from(name), i);
        }

        let model = &mut self.models[model_id];
        self.pin_groups
            .push(model.register_pin_group(id, name, pnames, endianness)?);
        Ok(&self.pin_groups[id])
    }

    pub fn get_pin_data(&self, model_id: usize, names: &Vec<String>) -> Result<u32, Error> {
        let mut data = 0;
        for n in names.iter().rev() {
            let p = self._get_pin(model_id, n)?;
            data = (data << 1) + *p.data.read().unwrap();
        }
        Ok(data as u32)
    }

    pub fn get_pin_reset_data(&self, model_id: usize, names: &Vec<String>) -> Result<u32, Error> {
        let mut rdata = 0;
        for n in names.iter().rev() {
            let p = self._get_pin(model_id, n)?;
            rdata = (rdata << 1) + p.reset_data.unwrap_or(0);
        }
        Ok(rdata as u32)
    }

    pub fn reset_pin_names(&mut self, model_id: usize, names: &Vec<String>) -> Result<(), Error> {
        for n in names.iter() {
            let p = self._get_mut_pin(model_id, n)?;
            p.reset();
        }
        Ok(())
    }

    pub fn set_pin_data(
        &mut self,
        model_id: usize,
        names: &Vec<String>,
        data: u32,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        self.data_fits_in_pins(names, data)?;

        let mut d = data;
        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for n in names.iter() {
            let p = self._get_mut_pin(model_id, n)?;
            p.set_data(((d & 0x1) & (m & 0x1)) as u8)?;
            d = d >> 1;
            m = m >> 1;
        }
        Ok(())
    }

    pub fn get_pin_actions(&self, model_id: usize, names: &Vec<String>) -> Result<Vec<PinActions>, Error> {
        let mut retn: Vec<PinActions> = vec!();
        for n in names.iter() {
            let p = self._get_pin(model_id, n)?;
            retn.push(p.action.read().unwrap().clone());
        }
        Ok(retn)
    }

    pub fn get_pin_reset_actions(
        &self,
        model_id: usize,
        names: &Vec<String>,
    ) -> Result<Vec<PinActions>, Error> {
        let mut retn: Vec<PinActions> = vec!();
        for n in names.iter() {
            let p = self._get_pin(model_id, n)?;
            retn.push(p.reset_action.clone().unwrap_or(PinActions::HighZ));
        }
        Ok(retn)
    }

    pub fn set_pin_actions(
        &mut self,
        model_id: usize,
        names: &Vec<String>,
        action: PinActions,
        data: Option<u32>,
        mask: Option<usize>,
        grp_id: Option<usize>,
    ) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_pin_data(model_id, names, d, mask)?;
        }

        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        let mut nodes: Vec<Node> = vec!();
        let mut actions: Vec<String> = vec!();
        for (_i, n) in names.iter().enumerate() {
            let p = self._get_mut_pin(model_id, n)?;
            {
                let mut pin_action = p.action.write().unwrap();

                if m & 0x1 == 1 {
                    {
                        let mut data = p.data.write().unwrap();
                        match action {
                            PinActions::DriveHigh | PinActions::VerifyHigh => *data = 1,
                            PinActions::DriveLow | PinActions::VerifyLow => *data = 0,
                            _ => {}
                        }
                    }
                    *pin_action = action.apply_state(*p.data.read().unwrap());
                } else {
                    *pin_action = PinActions::HighZ;
                }
                m >>= 1;
            }

            actions.push(p.action.read().unwrap().as_sym());
            nodes.push(crate::node!(PinAction, p.id, p.action.read().unwrap().as_sym(), None));
        }
        if let Some(i) = grp_id {
            let n_id = crate::TEST.push_and_open(crate::node!(PinGroupAction, i, actions, None));
            crate::TEST.append(&mut nodes);
            crate::TEST.close(n_id)?;
        } else {
            crate::TEST.append(&mut nodes);
        }
        Ok(())
    }

    pub fn set_per_pin_actions(
        &mut self,
        model_id: usize,
        names: &Vec<String>,
        actions: &Vec<PinActions>,
        mask: Option<usize>,
    ) -> Result<(), Error> {
        if names.len() != actions.len() {
            return Err(Error::new(&format!(
                "PinActions of length {} must match width {}!",
                actions.len(),
                names.len()
            )))
        }

        let mut m = (mask.unwrap_or(!(0 as usize))) as u32;
        for (i, n) in names.iter().enumerate() {
            let p = self._get_mut_pin(model_id, n)?;
            {
                let mut pin_action = p.action.write().unwrap();

                if m & 0x1 == 1 {
                    // If a single action was given, apply that to all pins
                    if actions.len() == 1 {
                        {
                            let mut data = p.data.write().unwrap();
                            match actions[0] {
                                PinActions::DriveHigh | PinActions::VerifyHigh => *data = 1,
                                PinActions::DriveLow | PinActions::VerifyLow => *data = 0,
                                _ => {}
                            }
                        }
                        *pin_action = actions[0].apply_state(*p.data.read().unwrap());
                    } else {
                        {
                            let mut data = p.data.write().unwrap();
                            match actions[i] {
                                PinActions::DriveHigh | PinActions::VerifyHigh => *data = 1,
                                PinActions::DriveLow | PinActions::VerifyLow => *data = 0,
                                _ => {}
                            }
                        }
                        *pin_action = actions[i].apply_state(*p.data.read().unwrap());
                    }
                } else {
                    *pin_action = PinActions::HighZ;
                }
            }
            m >>= 1;

            crate::TEST.push(crate::node!(PinAction, p.id, p.action.read().unwrap().as_sym(), None));
        }
        Ok(())
    }

    /// Given a group/collection of pin names, verify:
    ///     * Each pin exist
    ///     * Each pin is unique (no duplicate pins) AND it points to a unique physical pin. That is, each pin is unique after resolving aliases.
    /// If all the above is met, we can group/collect these names.
    pub fn verify_names(&self, model_id: usize, names: &Vec<String>) -> Result<Vec<String>, Error> {
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
                        for _name_str in grp.pin_names.iter() {
                            if physical_names.contains(_name_str) {
                                return Err(Error::new(&format!("Can not collect pin '{}' from regex /{}/ because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", name_str, regex_str, _name_str)));
                            }
                        }
                        _pin_names.extend(grp.pin_names.clone())
                    }
                }
                _pin_names.sort();
                physical_names.extend(_pin_names);
            } else if let Some(p) = self.resolve_to_physical_pin(model_id, pin_name) {
                if physical_names.contains(&p.name) {
                    return Err(Error::new(&format!("Can not collect pin '{}' because it (or an alias of it) has already been collected (resolves to physical pin '{}')!", pin_name, p.name)));
                }
                if let Some(p) = self.get_pin_group(model_id, pin_name) {
                    physical_names.extend_from_slice(&p.pin_names);
                }
            } else {
                return Err(Error::new(&format!(
                    "Can not collect pin '{}' because it does not exist!",
                    pin_name
                )));
            }
        }
        Ok(physical_names.clone())
    }

    pub fn collect(
        &mut self,
        model_id: usize,
        names: Vec<String>,
        endianness: Option<Endianness>,
    ) -> Result<PinCollection, Error> {
        let pnames = self.verify_names(model_id, &names)?;
        Ok(PinCollection::new(model_id, &pnames, endianness))
    }

    pub fn pin_names_contain(
        &self,
        model_id: usize,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<bool, Error> {
        let result = self.find_in_names(model_id, names, query_name)?.is_some();
        Ok(result)
    }

    pub fn find_in_names(
        &self,
        model_id: usize,
        names: &Vec<String>,
        query_name: &str,
    ) -> Result<Option<usize>, Error> {
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
            Err(Error::new(&format!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            )))
        }
    }

    /// Given a pin or alias name, finds either its name or alias in the group.
    pub fn index_of(
        &self,
        model_id: usize,
        name: &str,
        query_name: &str,
    ) -> Result<Option<usize>, Error> {
        if !self.models[model_id].pin_groups.contains_key(name) {
            // Pin group doesn't exists. Raise an error.
            return Err(Error::new(&format!(
                "Group {} does not exists! Cannot lookup index for {} in this group!",
                name, query_name
            )));
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
            Err(Error::new(&format!(
                "The query name {} does not exists! Cannot check this query's groups!",
                query_name
            )))
        }
    }

    pub fn resolve_to_physical_pin(&self, model_id: usize, name: &str) -> Option<&Pin> {
        if let Some(grp) = self.get_pin_group(model_id, name) {
            if let Some(physical_pin) = self.get_pin(model_id, &grp.pin_names[0]) {
                return Option::Some(physical_pin);
            }
        }
        Option::None
    }

    pub fn resolve_to_mut_physical_pin(&mut self, model_id: usize, name: &str) -> Option<&mut Pin> {
        let n;
        match self.get_pin_group(model_id, name) {
            Some(grp) => {
                n = grp.pin_names[0].clone();
            }
            None => return Option::None,
        }
        self.get_mut_pin(model_id, &n)
    }

    pub fn _resolve_to_physical_pin(&self, model_id: usize, name: &str) -> Result<&Pin, Error> {
        match self.resolve_to_physical_pin(model_id, name) {
            Some(p) => Ok(p),
            None => Err(Error::new(&format!("Cannot find physical pin '{}'!", name))),
        }
    }

    pub fn resolve_pin_names(
        &self,
        model_id: usize,
        names: &Vec<String>,
    ) -> Result<Vec<String>, Error> {
        let mut physical_names: Vec<String> = vec![];
        for (_i, n) in names.iter().enumerate() {
            let p = self._resolve_to_physical_pin(model_id, n)?;
            physical_names.push(p.name.clone());
        }
        Ok(physical_names)
    }

    pub fn data_fits_in_pins(&mut self, pins: &Vec<String>, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(pins.len() as u32) - 1) {
            Err(Error::new(&format!(
                "Data {} does not fit in Pin collection of size {} - Cannot set data!",
                data,
                pins.len()
            )))
        } else {
            Ok(())
        }
    }

    pub fn verify_data_fits(&mut self, width: u32, data: u32) -> Result<(), Error> {
        let two: u32 = 2;
        if data > (two.pow(width) - 1) {
            Err(Error::new(&format!(
                "Data {} does not fit in pins with width of {}!",
                data, width
            )))
        } else {
            Ok(())
        }
    }

    pub fn verify_action_string_fits(
        &self,
        width: u32,
        action_string: &Vec<u8>,
    ) -> Result<(), Error> {
        if action_string.len() != (width as usize) {
            Err(Error::new(&format!(
                "Action string of length {} must match width {}!",
                action_string.len(),
                width
            )))
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
    ) -> Result<bool, Error> {
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
    ) -> Result<Vec<&Pin>, Error> {
        let mut retn: Vec<&Pin> = vec![];
        let grp = self._get_pin_group(model_id, name)?;
        for p in grp.pin_names.iter() {
            retn.push(self._get_pin(model_id, p)?);
        }
        Ok(retn)
    }

    pub fn _resolve_groups_to_physical_pin_ids(&self, pins: &Vec<(usize, String)>) -> Result<Vec<Vec<usize>>, Error> {
        let mut retn: Vec<Vec<usize>> = vec!();
        for lookup in pins.iter() {
            let ppins = self._resolve_group_to_physical_pins(lookup.0, &lookup.1)?;
            retn.push(ppins.iter().map(|p| p.id).collect::<Vec<usize>>());
        }
        Ok(retn)
    }

    pub fn _resolve_to_flattened_pins(&self, pins: &Vec<(usize, String)>) -> Result<Vec<&Pin>, Error> {
        let mut retn: Vec<&Pin> = vec!();
        for lookup in pins.iter() {
            let mut ppins = self._resolve_group_to_physical_pins(lookup.0, &lookup.1)?;
            retn.append(&mut ppins);
        }
        Ok(retn)
    }

    /// Given a pin group name and model ID, converts it to a tuple containing:
    ///  [0] -> Vec<usize> containing the physical pin IDs of the pins in this group
    ///  [1] -> usize -> the resolved pin group ID
    pub fn pin_group_to_ids(&self, model_id: usize, pin_grp_name: &str) -> Result<(Vec<usize>, usize), Error> {
        let p_ids: Vec<usize> = self._resolve_group_to_physical_pins(
            model_id,
            pin_grp_name
        )?.iter().map( |p| p.id).collect();
        Ok((p_ids, self._get_pin_group(model_id, pin_grp_name)?.id))
    }
}

#[derive(Debug, Clone)]
pub struct StateTracker {
    // pins: IndexMap<String, Vec<(PinActions, u8)>>,
    pins: IndexMap<String, Vec<String>>,
    model_id: usize,
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
                        states.push(r.as_char().to_string());
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
                    pins.insert(
                        phys.name.clone(),
                        {
                            if let Some(r) = phys.reset_action.as_ref() {
                                vec!(r.as_char().to_string())
                            } else {
                                vec!("Z".to_string())
                            }
                        }
                    );
                }
            }
        }
        Self {
            pins: pins,
            model_id: model_id,
        }
    }

    /// Given a physical pin name, action, and data, updates the state appropriately
    pub fn update(
        &mut self,
        grp_id: usize,
        actions: &Vec<String>,
        dut: &Dut,
    ) -> Result<(), Error> {
        for (i, physical_pin) in dut.pin_groups[grp_id].pin_names.iter().enumerate() {
            let p = dut._get_pin(self.model_id, &physical_pin)?;
            // Check for the header pin in the aliases
            if let Some(states) = self.pins.get_mut(physical_pin) {
                states[0] = actions[i].clone();
                continue;
            }

            // Check for the header pin in the groups
            for (grp, offset) in p.groups.iter() {
                if let Some(states) = self.pins.get_mut(grp) {
                    states[*offset] = actions[i].clone();
                    continue;
                }
            }

            // Check for the header pin in the aliases
            for alias in p.aliases.iter() {
                if let Some(states) = self.pins.get_mut(alias) {
                    states[0] = actions[i].clone();
                    continue;
                }
            }
            // return Err(Error::new(&format!(
            //     "Could not resolve physical pin {} to any pins in header {}",
            //     physical_pin,
            //     self.pins
            //         .keys()
            //         .map(|n| n.to_string())
            //         .collect::<Vec<String>>()
            //         .join(", ")
            // )));
        }
        Ok(())
    }

    /// Processes the current state into a vector of 'state strings', where each string corresponds to a tester representation of the actions and data.
    /// E.g.: 'porta': [(PinAction::Drive), 1, (PinAction::HighZ, 0)], 'clk': [(PinAction::Drive), 1], 'reset': [(PinAction::Verify), 0]
    ///     => ['1Z', '1', 'L']
    /// If a header was given, the order will be identical to that from the header. If no header was given, the order will be whatever order was when the default
    /// pins were collected.
    pub fn to_symbols(&self, target: String, _dut: &Dut, t: &super::timesets::timeset::Timeset) -> Result<Vec<String>, Error> {
        let mut syms: Vec<String> = vec!();
        for (_n, states) in self.pins.iter() {
            let mut s: Vec<String> = vec!();
            for action in states.iter() {
                s.push(t._resolve_pin_action(target.clone(), &PinActions::from_delimiter_optional(&action)?)?.to_string());
            }
            syms.push(s.join(""));
        }
        Ok(syms)
    }

    pub fn names(&self) -> Vec<String> {
        self.pins
            .keys()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
    }

    pub fn contains_action(&self, action: PinActions) -> bool {
        for (_pin, actions) in self.pins.iter() {
            if actions.iter().any(|a| a.to_string() == action.as_char().to_string()) {
                return true;
            }
        }
        false
    }
}
