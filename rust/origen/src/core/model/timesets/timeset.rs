extern crate num;
use crate::error::Error;
use eval;
use indexmap::map::IndexMap;
use crate::core::dut::Dut;
use std::collections::HashMap;
use super::super::pins::pin::{PinActions, ResolvePinActions};
use super::super::pins::pin::Resolver as PinActionsResolver;
use crate::core::tester::TesterSource;

pub fn default_resolver() -> PinActionsResolver {
    let mut map = PinActionsResolver::new();
    map.update_mapping(PinActions::DriveHigh, "1".to_string());
    map.update_mapping(PinActions::DriveLow, "0".to_string());
    map.update_mapping(PinActions::VerifyHigh, "H".to_string());
    map.update_mapping(PinActions::VerifyLow, "L".to_string());
    map.update_mapping(PinActions::Capture, "C".to_string());
    map.update_mapping(PinActions::HighZ, "Z".to_string());
    map
}

pub struct SimpleTimeset {
    pub name: String,
    pub period: Option<String>,
    pub default_period: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Timeset {
    pub name: String,
    pub model_id: usize,
    pub id: usize,
    pub period_as_string: Option<String>,
    pub default_period: Option<f64>,
    pub wavetable_ids: IndexMap<String, usize>,
    pub pin_action_resolvers: IndexMap<String, PinActionsResolver>,

    active_wavetable: Option<String>,
}

impl Timeset {
    pub fn new(
        model_id: usize,
        id: usize,
        name: &str,
        period_as_string: Option<Box<dyn std::string::ToString>>,
        default_period: Option<f64>,
        targets: Vec<&TesterSource>,
    ) -> Self {
        Timeset {
            model_id: model_id,
            id: id,
            name: String::from(name),
            period_as_string: match period_as_string {
                Some(p) => Some(p.to_string()),
                None => None,
            },
            default_period: default_period,
            wavetable_ids: IndexMap::new(),
            active_wavetable: Option::None,
            //allow_implicit_pin_lookups: false,
            pin_action_resolvers: {
                let mut i = IndexMap::new();
                for target in targets {
                    match target {
                        TesterSource::External(tester_name) => {
                            i.insert(tester_name.to_string(), default_resolver());
                        },
                        TesterSource::Internal(t) => {
                            if let Some(r) = t.pin_action_resolver() {
                                i.insert(t.id(), r);
                            } else {
                                i.insert(t.id(), default_resolver());
                            }
                        }
                    }
                }
                i
            }
        }
    }

    pub fn eval_str(&self) -> String {
        let default = String::from("period");
        let p = self.period_as_string.as_ref().unwrap_or(&default);
        p.clone()
    }

    pub fn activate_wavetable(&mut self, wtbl_name: &str) -> Result<(), Error> {
        if self.wavetable_ids.contains_key(wtbl_name) {
            self.active_wavetable = Some(wtbl_name.to_string());
            Ok(())
        } else {
            Err(Error::new(&format!(
                "Timeset {} does not have a wavetable named {}!",
                self.name,
                wtbl_name
            )))
        }
    }

    pub fn clear_active_wavetable(&mut self) {
        self.active_wavetable = Option::None;
    }

    pub fn active_wavetable(&self, dut: Dut) -> Option<String> {
        self.active_wavetable.clone()
    }

    pub fn eval(&self, current_period: Option<f64>) -> Result<f64, Error> {
        let default = String::from("period");
        let p = self.period_as_string.as_ref().unwrap_or(&default);
        let err = &format!(
            "Could not evaluate Timeset {}'s expression: '{}'",
            self.name, p
        );
        if current_period.is_none() && self.default_period.is_none() {
            return Err(Error::new(&format!("No current timeset period set! Cannot evaluate timeset '{}' without a current period as it does not have a default period!", self.name)));
        }
        match eval::Expr::new(p)
            .value(
                "period",
                current_period.unwrap_or(self.default_period.unwrap()),
            )
            .exec()
        {
            Ok(val) => match val.as_f64() {
                Some(v) => Ok(v),
                None => Err(Error::new(err)),
            },
            Err(_e) => Err(Error::new(err)),
        }
    }

    pub fn register_wavetable(&mut self, id: usize, name: &str) -> Result<Wavetable, Error> {
        let w = Wavetable::new(self.model_id, self.id, id, name)?;
        self.wavetable_ids.insert(String::from(name), id);
        Ok(w)
    }

    pub fn get_wavetable_id(&self, name: &str) -> Option<usize> {
        match self.wavetable_ids.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }

    pub fn contains_wavetable(&self, name: &str) -> bool {
        self.wavetable_ids.contains_key(name)
    }
}

impl Default for Timeset {
    fn default() -> Self {
        Self::new(0, 0, "dummy", Option::None, Option::None, vec![])
    }
}

impl ResolvePinActions for Timeset {
    fn pin_action_resolver(&self, target: String) -> &PinActionsResolver {
        &self.pin_action_resolvers[&target]
    }

    fn mut_pin_action_resolver(&mut self, target: String) -> &mut PinActionsResolver {
        &mut self.pin_action_resolvers[&target]
    }
}

// Structuring the WaveTable for ATE generation which, when a pin is changed, will look up here what that change 'means' and
// what to actually put in the output.
// So, given a known Timeset and Wavetable, allow for a quick lookup of an event from a given PinGroup name.
#[derive(Debug, Clone)]
pub struct Wavetable {
    pub name: String,
    pub model_id: usize,
    pub timeset_id: usize,
    pub id: usize,
    pub period: Option<String>,

    // The 'waveforms', 'signals', and 'event' from the STIL specification is mashed together
    // in an 'event_map', whose key is a PinGroup name and whose values are a list of events associated
    // with that PinGroup.
    // Note that events are Event-IDs, which can be looked up from the DUT.
    //pub wave_ids: IndexMap<String, usize>
    pub wave_group_ids: IndexMap<String, usize>,

    /// Stores the wave <-> physical relationship, after ID resolution
    applied_waves: HashMap<usize, HashMap<String, usize>>,
    inherited_wavetable_ids: Vec<usize>,
}

impl Wavetable {
    pub fn new(model_id: usize, timeset_id: usize, id: usize, name: &str) -> Result<Self, Error> {
        Ok(Self {
            name: String::from(name),
            model_id: model_id,
            timeset_id: timeset_id,
            id: id,
            period: Option::None,
            wave_group_ids: IndexMap::new(),
            applied_waves: HashMap::new(),
            inherited_wavetable_ids: Vec::new(),
        })
    }

    pub fn apply_wave_to(&mut self, physical_pin_id: usize, wave_indicator: &str, wave_id: usize) {
        if let Some(pin_waves) = self.applied_waves.get_mut(&physical_pin_id) {
            pin_waves.insert(wave_indicator.to_string(), wave_id);
        } else {
            let mut pin_waves = HashMap::new();
            pin_waves.insert(wave_indicator.to_string(), wave_id);
            self.applied_waves.insert(physical_pin_id, pin_waves);
        }
    }

    /// Returns all the wave IDs, including inherited ones, which is included in the pin list and
    /// has an indicator in the indicator list.
    /// If the pin list is empty, then all pins are returned. Likewise with the indicator list.
    pub fn applied_waves(&self, dut: &Dut, pins: &Vec<(usize, String)>, indicators: &Vec<String>) -> Result<HashMap<usize, HashMap<String, usize>>, Error>{
        let __pins;
        if pins.len() > 1 {
            let t: Vec<usize> = vec!();
            for (model_id, pname) in pins.iter() {
                dut._get_pin(*model_id, &pname)?.id;
            }
            __pins = t;
        } else {
            __pins = self.applied_waves.iter().map(|(k, _)| *k).collect();
        }

        let mut retn: HashMap<usize, HashMap<String, usize>> = HashMap::new();
        for p in __pins.iter() {
            if let Some(wids) = self.wave_ids_for(dut, *p, indicators) {
                retn.insert(*p, wids);
            }
        }
        Ok(retn)
    }

    pub fn wave_ids_for(&self, dut: &Dut, pin_id: usize, indicators: &Vec<String>) -> Option<HashMap<String, usize>> {
        let mut retn: HashMap<String, usize> = HashMap::new();
        if let Some(waves) = self.applied_waves.get(&pin_id) {
            retn.extend(waves.clone());
        }
        for parent_id in self.inherited_wavetable_ids.iter().rev() {
            let parent = &dut.wavetables[*parent_id];
            // Notes here:
            //  Could potentially be an inheritance chain, so need to use the
            //      the 'waves_for' method over just checking applied_waves structure
            //  Any circular inheritance structures should've been weeded out
            //      during initialization, so not checking here.
            //  We're only checking one pin at a time, so the result will be
            //      a vector of size 1 with either Option::None or Option<wave_id>
            //  Either way, we can just stick this directly into the return
            // retn.insert(parent.wave_ids_for(dut, &vec!(*pid))[0]);
            let mut t = parent.wave_ids_for(dut, pin_id, indicators).unwrap_or(HashMap::new());
            t.extend(retn);
            retn = t;
        }
        if retn.keys().len() > 0 {
            Some(retn)
        } else {
            None
        }
    }

    pub fn get_wave_group_id(&self, name: &str) -> Option<usize> {
        match self.wave_group_ids.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }

    pub fn contains_wave_group(&self, name: &str) -> bool {
        self.wave_group_ids.contains_key(name)
    }

    // At some point, add some preliminary checking of the period to weed out some gross failures.
    pub fn set_period(
        &mut self,
        period: Option<Box<dyn std::string::ToString>>,
    ) -> Result<(), Error> {
        self.period = match period {
            Some(p) => Some(p.to_string()),
            None => None,
        };
        Ok(())
    }

    pub fn register_wave_group(
        &mut self,
        id: usize,
        name: &str,
        derived_from: Option<Vec<usize>>,
    ) -> Result<WaveGroup, Error> {
        let wgrp = WaveGroup::new_from_wavetable(self, id, name, derived_from)?;
        self.wave_group_ids.insert(String::from(name), id);
        Ok(wgrp)
    }

    pub fn eval(&self, current_period: Option<f64>) -> Result<Option<f64>, Error> {
        let default = String::from("period");
        let p = self.period.as_ref().unwrap_or(&default);
        let err = &format!(
            "Could not evaluate Wavetable {}'s expression: '{}'",
            self.name, p
        );
        if current_period.is_none() && self.period.is_none() {
            return Ok(Option::None);
        }
        if current_period.is_none()
            && eval::Expr::new(self.period.as_ref().unwrap())
                .exec()
                .is_err()
        {
            return Err(Error::new(&format!("No current period set for wavetable {} and cannot evaluate period '{}' as a numeric!", self.name, self.period.as_ref().unwrap())));
        }
        match eval::Expr::new(p).exec() {
            Ok(val) => match val.as_f64() {
                Some(v) => Ok(Some(v)),
                None => Err(Error::new(err)),
            },
            Err(_e) => Err(Error::new(err)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaveGroup {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub id: usize,
    pub name: String,
    pub wave_ids: IndexMap<String, usize>,
    pub derived_from: Option<Vec<usize>>,
}

impl WaveGroup {
    // If a wavetable is given, extract the IDs from there instead of tearing it apart upstream just to put it back together again here.
    pub fn new_from_wavetable(
        wavetable: &Wavetable,
        id: usize,
        name: &str,
        derived_from: Option<Vec<usize>>,
    ) -> Result<Self, Error> {
        let wgrp = WaveGroup {
            model_id: wavetable.model_id,
            timeset_id: wavetable.timeset_id,
            wavetable_id: wavetable.id,
            id: id,
            name: String::from(name),
            wave_ids: IndexMap::new(),
            derived_from: derived_from,
        };
        Ok(wgrp)
    }

    pub fn register_wave(&mut self, id: usize, indicator: &str) -> Result<Wave, Error> {
        let w = Wave::new_from_wave_group(self, id, indicator)?;
        self.wave_ids.insert(indicator.to_string(), id);
        Ok(w)
    }

    pub fn get_wave_id(&self, name: &str) -> Option<usize> {
        match self.wave_ids.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }

    pub fn contains_wave(&self, name: &str) -> bool {
        self.wave_ids.contains_key(name)
    }
}

#[derive(Debug, Clone)]
pub struct Wave {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub wave_group_id: usize,
    pub wave_id: usize,
    pub events: Vec<usize>,
    pub applied_pin_ids: Vec<usize>,
    pub indicator: String,
}

impl Wave {
    pub fn new(
        model_id: usize,
        timeset_id: usize,
        wavetable_id: usize,
        wave_group_id: usize,
        wave_id: usize,
        indicator: &str,
    ) -> Result<Self, Error> {
        Ok(Wave {
            model_id: model_id,
            timeset_id: timeset_id,
            wavetable_id: wavetable_id,
            wave_group_id: wave_group_id,
            wave_id: wave_id,
            events: vec![],
            applied_pin_ids: vec![],
            indicator: String::from(indicator),
        })
    }

    pub fn new_from_wave_group(
        wgrp: &WaveGroup,
        id: usize,
        indicator: &str,
    ) -> Result<Self, Error> {
        Ok(Wave {
            model_id: wgrp.model_id,
            timeset_id: wgrp.timeset_id,
            wavetable_id: wgrp.wavetable_id,
            wave_group_id: wgrp.id,
            wave_id: id,
            events: vec![],
            applied_pin_ids: vec![],
            indicator: String::from(indicator),
        })
    }

    pub fn set_indicator(&mut self, indicator: &str) -> Result<(), Error> {
        // At some point, add some error checking here, preferably from feedback from the current platform (allowed indicators, etc.)
        self.indicator = String::from(indicator);
        Ok(())
    }

    pub fn get_event_id(&self, event_index: usize) -> Option<usize> {
        if event_index < self.events.len() {
            Some(self.events[event_index])
        } else {
            None
        }
    }

    //pub fn register_event(&mut self, id: usize) -> Result<(), Error> {
    pub fn push_event(
        &mut self,
        e_id: usize,
        at: Box<dyn std::string::ToString>,
        unit: Option<String>,
        action: &str,
    ) -> Result<Event, Error> {
        let e = Event::new(
            self.wavetable_id,
            self.wave_id,
            e_id,
            self.events.len(),
            at,
            unit,
            action,
        )?;
        self.events.push(e_id);
        Ok(e)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EventActions {
    DriveHigh,
    DriveLow,
    VerifyHigh,
    VerifyLow,
    VerifyZ,
    HighZ,
    Capture,
}

impl EventActions {
    pub fn from_str(s: &str) -> Result<EventActions, Error> {
        match s {
            "DriveHigh" => Ok(EventActions::DriveHigh),
            "DriveLow" => Ok(EventActions::DriveLow),
            "VerifyHigh" => Ok(EventActions::VerifyHigh),
            "VerifyLow" => Ok(EventActions::VerifyLow),
            "VerifyZ" => Ok(EventActions::VerifyZ),
            "HighZ" => Ok(EventActions::HighZ),
            "Capture" => Ok(EventActions::Capture),
            _ => Err(Error::new(&format!("Unsupported Event Action: '{}'", s))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EventActions::DriveHigh => "DriveHigh",
            EventActions::DriveLow => "DriveLow",
            EventActions::VerifyHigh => "VerifyHigh",
            EventActions::VerifyLow => "VerifyLow",
            EventActions::VerifyZ => "VerifyZ",
            EventActions::HighZ => "HighZ",
            EventActions::Capture => "Capture",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub wavetable_id: usize,
    pub wave_id: usize,
    pub event_id: usize,
    pub event_index: usize,
    pub action: String,
    pub at: String,
    pub unit: Option<String>,
}

impl Event {
    fn new(
        wavetable_id: usize,
        wave_id: usize,
        e_id: usize,
        e_i: usize,
        at: Box<dyn std::string::ToString>,
        unit: Option<String>,
        action: &str,
    ) -> Result<Self, Error> {
        let _temp = EventActions::from_str(action)?;
        let e = Event {
            wavetable_id: wavetable_id,
            wave_id: wave_id,
            event_id: e_id,
            event_index: e_i,
            at: at.to_string(),
            unit: unit,
            action: action.to_string(),
        };
        Ok(e)
    }

    pub fn eval(
        &self,
        dut: &super::super::super::dut::Dut,
        period: Option<f64>,
    ) -> Result<f64, Error> {
        // Try to evaluate the wavetable's period.
        let err = &format!("Could not evaluate event expression '{}'", self.at);
        let _period;
        {
            _period = dut.wavetables[self.wavetable_id].eval(period)?;
        }

        // Try to evaluate the event
        match eval::Expr::new(&self.at).value("period", _period).exec() {
            Ok(val) => match val.as_f64() {
                Some(v) => Ok(v),
                None => Err(Error::new(err)),
            },
            Err(_e) => Err(Error::new(err)),
        }
    }

    pub fn set_action(&mut self, action: &str) -> Result<(), Error> {
        let _temp = EventActions::from_str(action)?;
        self.action = action.to_string();
        Ok(())
    }
}

impl Dut {
    pub fn apply_wave_id_to_pins(&mut self, wave_id: usize, pins: &Vec<(usize, String)>) -> Result<(), Error> {
        let (wtbl_id, wave_indicator);
        {
            let wave = &self.waves[wave_id];
            wtbl_id = wave.wavetable_id;
            wave_indicator = wave.indicator.clone();
        }

        let physical_pin_ids: Vec<Vec<usize>> = self._resolve_groups_to_physical_pin_ids(&pins)?;
        for ppin_ids in physical_pin_ids.iter() {
            for ppin_id in ppin_ids.iter() {
                    {
                    let wtbl = &mut self.wavetables[wtbl_id];
                    wtbl.apply_wave_to(*ppin_id, &wave_indicator, wave_id);
                }
                {
                    let wave = &mut self.waves[wave_id];
                    wave.applied_pin_ids.push(*ppin_id);
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test() {
    let t = Timeset::new(0, 0, "t1", Some(Box::new("1.0 + 1")), Option::None, vec![]);
    assert!(t.eval(None).is_err());

    let t = Timeset::new(0, 0, "t1", Some(Box::new("period")), Some(1.0 as f64), vec![]);
    assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.0 as f64);

    let t = Timeset::new(
        0,
        0,
        "t1",
        Some(Box::new("period + 0.25")),
        Some(1.0 as f64),
        vec![],
    );
    assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.25 as f64);
}
