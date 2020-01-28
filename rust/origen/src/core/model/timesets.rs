pub mod timeset;

use super::Model;
use timeset::{Timeset, Wavetable, Wave, WaveGroup, Event};
use crate::error::Error;
use super::super::dut::Dut;

/// Returns an Origen Error with pre-formatted message comlaining that
/// someting already exists.
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

impl Model {
  pub fn add_timeset(&mut self, model_id: usize, instance_id: usize, name: &str, period: Option<Box<dyn std::string::ToString>>, default_period: Option<f64>) -> Result<Timeset, Error> {
    let t = Timeset::new(model_id, instance_id, name, period, default_period);
    self.timesets.insert(String::from(name), instance_id);
    Ok(t)
  }

  pub fn get_timeset_id(&self, name: &str) -> Option<usize> {
    match self.timesets.get(name) {
      Some(t) => Some(*t),
      None => None,
    }
  }

  pub fn len(&self) -> usize {
    self.timesets.len()
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
  ) -> Result<&Timeset, Error> {
    let id;
    {
      id = self.timesets.len();
    }

    let t;
    {
      let model= self.get_mut_model(model_id)?;
      if model.contains_timeset(name) {
        return duplicate_error!("timeset", model.name, name);
      }
  
      t = model.add_timeset(
        model_id,
        id,
        name,
        period,
        default_period,
      )?;
    }
    self.timesets.push(t);
    Ok(&self.timesets[id])
  }
  
  pub fn get_timeset(&self, model_id: usize, name: &str) -> Option<&Timeset> {
    if let Some(t) = self.get_model(model_id).unwrap().get_timeset_id(name) {
      Some(&self.timesets[t])
    } else {
      Option::None
    }
  }

  pub fn get_mut_timeset(&mut self, model_id: usize, name: &str) -> Option<&mut Timeset> {
    if let Some(t) = self.get_model(model_id).unwrap().get_timeset_id(name) {
      Some(&mut self.timesets[t])
    } else {
      Option::None
    }
  }

  pub fn _get_timeset(&self, model_id: usize, name: &str) -> Result<&Timeset, Error> {
    match self.get_timeset(model_id, name) {
      Some(t) => Ok(t),
      None => backend_lookup_error!("timeset", name)
    }
  }

  pub fn _get_mut_timeset(&mut self, model_id: usize, name: &str) -> Result<&mut Timeset, Error> {
    match self.get_mut_timeset(model_id, name) {
      Some(t) => Ok(t),
      None => backend_lookup_error!("timeset", name)
    }
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
  
      w = t.register_wavetable(
        id,
        name
      )?;
    }
    self.wavetables.push(w);
    Ok(&self.wavetables[id])
  }

  pub fn get_wavetable(&self, timeset_id: usize, name: &str) -> Option<&Wavetable> {
    if let Some(w) = self.timesets[timeset_id].get_wavetable_id(name) {
      Some(&self.wavetables[w])
    } else {
      Option::None
    }
  }

  pub fn get_mut_wavetable(&mut self, timeset_id: usize, name: &str) -> Option<&mut Wavetable> {
    if let Some(w) = self.timesets[timeset_id].get_wavetable_id(name) {
      Some(&mut self.wavetables[w])
    } else {
      Option::None
    }
  }

  pub fn create_wave_group(&mut self, wavetable_id: usize, name: &str, derived_from: Option<Vec<usize>>) -> Result<&WaveGroup, Error> {
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
  
      wgrp = wtbl.register_wave_group(
        id,
        name,
        derived_from
      )?;
    }
    self.wave_groups.push(wgrp);
    Ok(&self.wave_groups[id])
  }

  pub fn create_wave(&mut self, wave_group_id: usize, indicator: &str) -> Result<&Wave, Error> {
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
  
      w = wgrp.register_wave(
        id,
        indicator
      )?;
    }
    self.waves.push(w);
    Ok(&self.waves[id])
  }

  pub fn get_wave_group(&self, wavetable_id: usize, name: &str) -> Option<&WaveGroup> {
    if let Some(wgrp_id) = self.wavetables[wavetable_id].get_wave_group_id(name) {
      Some(&self.wave_groups[wgrp_id])
    } else {
      Option::None
    }
  }

  pub fn get_mut_wave_group(&mut self, wavetable_id: usize, name: &str) -> Option<&mut WaveGroup> {
    if let Some(wgrp_id) = self.wavetables[wavetable_id].get_wave_group_id(name) {
      Some(&mut self.wave_groups[wgrp_id])
    } else {
      Option::None
    }
  }

  pub fn get_wave(&self, wave_group_id: usize, name: &str) -> Option<&Wave> {
    if let Some(w) = self.wave_groups[wave_group_id].get_wave_id(name) {
      Some(&self.waves[w])
    } else {
      Option::None
    }
  }

  pub fn get_mut_wave(&mut self, wave_group_id: usize, name: &str) -> Option<&mut Wave> {
    if let Some(w) = self.wave_groups[wave_group_id].get_wave_id(name) {
      Some(&mut self.waves[w])
    } else {
      Option::None
    }
  }

  pub fn create_event(&mut self, wave_id: usize, at: Box<dyn std::string::ToString>, unit: Option<String>, action: &str) -> Result<&Event, Error> {
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

  /// Gets an event at index 'event_index' from a wave.
  /// Note: the indices are from the wave, NOT from the toplevel DUT. Using dut.waves_events[event_index] will
  ///   probably return the wrong event, which may not even be on the wave in question.
  pub fn get_event(&self, wave_id: usize, event_index: usize) -> Option<&Event> {
    if let Some(e_id) = self.waves[wave_id].get_event_id(event_index) {
      Some(&self.wave_events[e_id])
    } else {
      Option::None
    }
  }

  pub fn get_mut_event(&mut self, wave_id: usize, event_index: usize) -> Option<&mut Event> {
    if let Some(e_id) = self.waves[wave_id].get_event_id(event_index) {
      Some(&mut self.wave_events[e_id])
    } else {
      Option::None
    }
  }
}
