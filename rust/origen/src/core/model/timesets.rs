pub mod timeset;

use super::super::dut::Dut;
use super::super::tester::TesterSource;
use super::Model;
use crate::error::Error;
use timeset::{Event, Timeset, Wave, WaveGroup, Wavetable};

/// Returns an Origen Error with pre-formatted message complaining that
/// something already exists.
#[macro_export]
macro_rules! duplicate_error {
    ($container:expr, $model_name:expr, $duplicate_name:expr) => {
        Err(Error::new(&format!(
            "The block '{}' already contains a(n) {} called '{}'",
            $model_name, $container, $duplicate_name
        )))
    };
}

/// Returns a error stating that the backend doesn't have an expected ID.
/// This signals a bug somewhere and should only be used when we're expecting
/// something to exists.
#[macro_export]
macro_rules! backend_lookup_error {
    ($container:expr, $name:expr) => {
        Err(Error::new(&format!(
            "Something has gone wrong, no {} exists with ID '{}'",
            $container, $name
        )))
    };
}

#[macro_export]
macro_rules! lookup_error {
    ($container:expr, $name:expr) => {
        Err(Error::new(&format!(
            "Could not find {} named {}!",
            $container, $name
        )))
    };
}

impl Model {
    pub fn add_timeset(
        &mut self,
        model_id: usize,
        instance_id: usize,
        name: &str,
        period: Option<Box<dyn std::string::ToString>>,
        default_period: Option<f64>,
        targets: Vec<&TesterSource>,
    ) -> Result<Timeset, Error> {
        let t = Timeset::new(model_id, instance_id, name, period, default_period, targets);
        self.timesets.insert(String::from(name), instance_id);
        Ok(t)
    }

    pub fn get_timeset_id(&self, name: &str) -> Option<usize> {
        match self.timesets.get(name) {
            Some(t) => Some(*t),
            None => None,
        }
    }

    pub fn contains_timeset(&self, name: &str) -> bool {
        self.timesets.contains_key(name)
    }
}

impl Dut {
    pub fn create_timeset(
        &mut self,
        model_id: usize,
        name: &str,
        period: Option<Box<dyn std::string::ToString>>,
        default_period: Option<f64>,
        targets: Vec<&TesterSource>,
    ) -> Result<&Timeset, Error> {
        let id;
        {
            id = self.timesets.len();
        }

        let t;
        {
            let model = self.get_mut_model(model_id)?;
            if model.contains_timeset(name) {
                return duplicate_error!("timeset", model.name, name);
            }

            t = model.add_timeset(model_id, id, name, period, default_period, targets)?;
        }
        self.timesets.push(t);
        Ok(&self.timesets[id])
    }

    pub fn create_wavetable(&mut self, timeset_id: usize, name: &str) -> Result<&Wavetable, Error> {
        let id;
        {
            id = self.wavetables.len();
        }

        let w;
        {
            let t = &mut self.timesets[timeset_id];
            if t.contains_wavetable(name) {
                return duplicate_error!("wavetable", t.name, name);
            }

            w = t.register_wavetable(id, name)?;
        }
        self.wavetables.push(w);
        Ok(&self.wavetables[id])
    }

    pub fn create_wave_group(
        &mut self,
        wavetable_id: usize,
        name: &str,
        derived_from: Option<Vec<usize>>,
    ) -> Result<&WaveGroup, Error> {
        let id;
        {
            id = self.wave_groups.len();
        }

        let wgrp;
        {
            let wtbl = &mut self.wavetables[wavetable_id];
            if wtbl.contains_wave_group(name) {
                return duplicate_error!("wave group", wtbl.name, name);
            }

            wgrp = wtbl.register_wave_group(id, name, derived_from)?;
        }
        self.wave_groups.push(wgrp);
        Ok(&self.wave_groups[id])
    }

    pub fn create_wave(
        &mut self,
        wave_group_id: usize,
        indicator: &str,
        derived_from: Option<Vec<String>>,
    ) -> Result<&Wave, Error> {
        let id;
        {
            id = self.waves.len();
        }

        let w;
        {
            let wgrp = &mut self.wave_groups[wave_group_id];
            if wgrp.contains_wave(indicator) {
                return duplicate_error!("wave", wgrp.name, indicator);
            }

            w = wgrp.register_wave(id, indicator)?;
        }
        self.waves.push(w);

        if let Some(bases) = derived_from {
            for base in bases.iter() {
                let base_wave: Wave;
                {
                    base_wave = self._get_cloned_wave(wave_group_id, base)?;
                }
                {
                    // Todo: Need to further define how wave inheritance should look.
                    // In the wave is independent, so we're recreating the events, not just storing a reference to the derived wave's events.
                    // We're also allowing one wave to come in an knock out the events of another, if it includes any events.
                    // So, we'll only overwrite events if the another wave includes them. But, there's no way to know this ahead of time without doing a second pass.
                    for e_id in base_wave.events.iter() {
                        let e = self.wave_events[*e_id].clone();
                        self.create_event(id, Box::new(e.at), e.unit, &e.action)?;
                    }
                }
                // If given, pull the following out of the wave:
                //  * pins
                //  * events
                // Todo - add waveform inheritance
                // let w = &mut self.waves[id];
                // w.pins = base_wave.pins.clone();
            }
        }
        Ok(&self.waves[id])
    }

    pub fn create_event(
        &mut self,
        wave_id: usize,
        at: Box<dyn std::string::ToString>,
        unit: Option<String>,
        action: &str,
    ) -> Result<&Event, Error> {
        let e_id;
        {
            e_id = self.wave_events.len();
        }

        let event;
        {
            let w = &mut self.waves[wave_id];
            event = w.push_event(e_id, at, unit, action)?;
        }
        self.wave_events.push(event);
        Ok(&self.wave_events[e_id])
    }
}
