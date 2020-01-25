extern crate num;
//use super::super::pins::pin::PinActions;
use crate::error::Error;
use eval;
use indexmap::map::IndexMap;

pub struct SimpleTimeset {
  pub name: String,
  pub period: Option<String>,
  pub default_period: Option<f64>
}

#[derive(Debug)]
pub struct Timeset {
  pub name: String,
  pub model_id: usize,
  pub id: usize,
  pub period_as_string: Option<String>,
  pub default_period: Option<f64>,
  pub wavetable_ids: IndexMap<String, usize>,
}

impl Timeset {
  pub fn new(model_id: usize, id: usize, name: &str, period_as_string: Option<Box<dyn std::string::ToString>>, default_period: Option<f64>) -> Self {
    Timeset {
      model_id: model_id,
      id: id,
      name: String::from(name),
      period_as_string: match period_as_string {
        Some(p) => Some(p.to_string()),
        None => None
      },
      default_period: default_period,
      wavetable_ids: IndexMap::new(),
    }
  }

  pub fn eval_str(&self) -> String {
    let default = String::from("period");
    let p = self.period_as_string.as_ref().unwrap_or(&default);
    p.clone()
  }

  pub fn eval(&self, current_period: Option<f64>)-> Result<f64, Error> {
    let default = String::from("period");
    let p = self.period_as_string.as_ref().unwrap_or(&default);
    let err = &format!("Could not evaluate Timeset {}'s expression: '{}'", self.name, p);
    if current_period.is_none() && self.default_period.is_none() {
      return Err(Error::new(&format!("No current timeset period set! Cannot evalate timeset '{}' without a current period as it does not have a default period!", self.name)));
    }
    match eval::Expr::new(p).value("period", current_period.unwrap_or(self.default_period.unwrap())).exec() {
      Ok(val) => {
        match val.as_f64() {
          Some(v) => Ok(v),
          None => Err(Error::new(err))
        }
      }
      Err(_e) => Err(Error::new(err))
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
    Self::new(0, 0, "dummy", Option::None, Option::None)
  }
}

// Structuring the WaveTable for ATE generation which, when a pin is changed, will look up here what that change 'means' and
// what to actually put in the output.
// So, given a known Timeset and Wavetable, allow for a quick lookup of an event from a given PinGroup name.
#[derive(Debug)]
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
  pub wave_ids: IndexMap<String, usize>
}

impl Wavetable {
  pub fn new(model_id: usize, timeset_id: usize, id: usize, name: &str) -> Result<Self, Error> {
    Ok(Self {
      name: String::from(name),
      model_id: model_id,
      timeset_id: timeset_id,
      id: id,
      period: Option::None,
      wave_ids: IndexMap::new(),
    })
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

  // At some point, add some preliminary checking of the period to weed out some gross failures.
  pub fn set_period(&mut self, period: Option<Box<dyn std::string::ToString>>) -> Result<(), Error> {
    self.period = match period {
      Some(p) => Some(p.to_string()),
      None => None
    };
    Ok(())
  }

  pub fn register_wave(&mut self, id: usize, name: &str) -> Result<Wave, Error> {
    let w = Wave::new(self.model_id, self.timeset_id, self.id, id, name)?;
    self.wave_ids.insert(String::from(name), id);
    Ok(w)
  }

  pub fn eval(&self, current_period: Option<f64>) -> Result<Option<f64>, Error> {
    let default = String::from("period");
    let p = self.period.as_ref().unwrap_or(&default);
    let err = &format!("Could not evaluate Wavetable {}'s expression: '{}'", self.name, p);
    if current_period.is_none() && self.period.is_none() {
      return Ok(Option::None);
    }
    if current_period.is_none() && eval::Expr::new(self.period.as_ref().unwrap()).exec().is_err() {
      return Err(Error::new(&format!("No current period set for wavetable {} and cannot evaluate period '{}' as a numeric!", self.name, self.period.as_ref().unwrap())));
    }
    match eval::Expr::new(p).exec() {
        Ok(val) => {
        match val.as_f64() {
          Some(v) => Ok(Some(v)),
          None => Err(Error::new(err))
        }
      }
      Err(_e) => Err(Error::new(err))
    }
  }

  pub fn get_waves_applied_to(&self, dut: &super::super::super::dut::Dut, pin: &str) -> Vec<String> {
    let mut rtn: Vec<String> = vec!();
    for (name, id) in self.wave_ids.iter() {
      let w = &dut.waves[*id];
      if w.pins.contains(&pin.to_string()) {
        rtn.push(name.clone());
      }
    }
    rtn
  }
}

#[derive(Debug)]
pub struct Wave {
  pub model_id: usize,
  pub timeset_id: usize,
  pub wavetable_id: usize,
  pub wave_id: usize,
  pub name: String,
  pub events: Vec<usize>,
  pub pins: Vec<String>,
  pub indicator: String,
}

impl Wave {
  pub fn new(model_id: usize, timeset_id: usize, wavetable_id: usize, wave_id: usize, name: &str) -> Result<Self, Error> {
    Ok(Wave {
      model_id: model_id,
      timeset_id: timeset_id,
      wavetable_id: wavetable_id,
      wave_id: wave_id,
      name: String::from(name),
      events: vec!(),
      pins: vec!(),
      indicator: String::from(""),
    })
  }

  pub fn apply_to(&mut self, pins: Vec<String>) -> Result<(), Error> {
    // At some point, add some error handling to ensure the pins exists, isn't already included, etc.
    // For now though, just adding it.
    self.pins.extend(pins.clone());
    Ok(())
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
  pub fn push_event(&mut self, e_id: usize, at: Box<dyn std::string::ToString>, unit: Option<String>, action: &str) -> Result<Event, Error> {
    let e = Event::new(self.wavetable_id, self.wave_id, e_id, self.events.len(), at, unit, action)?;
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
        _ => Err(Error::new(&format!(
            "Unsupported Event Action: '{}'",
            s
        ))),
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

#[derive(Debug)]
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
  fn new(wavetable_id: usize, wave_id: usize, e_id: usize, e_i: usize, at: Box<dyn std::string::ToString>, unit: Option<String>, action: &str) -> Result<Self, Error> {
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

  pub fn eval(&self, dut: &super::super::super::dut::Dut, period: Option<f64>) -> Result<f64, Error> {
    // Try to evaluate the wavetable's period.
    let err = &format!("Could not evaluate event expression '{}'", self.at);
    let _period;
    {
      _period = dut.wavetables[self.wavetable_id].eval(period)?;
    }

    // Try to evaluate the event
    match eval::Expr::new(&self.at).value("period", _period).exec() {
      Ok(val) => {
        match val.as_f64() {
          Some(v) => Ok(v),
          None => Err(Error::new(err))
        }
      }
      Err(_e) => Err(Error::new(err))
    }
  }
}

#[test]
fn test() {
  //let e = Event { action: PinActions::Drive, at: vec!() };
  // let t = Timeset::new("t1", Some(Box::new("1.0")), Option::None);
  // assert_eq!(t.eval(None).unwrap(), 1.0 as f64);

  let t = Timeset::new(0, 0, "t1", Some(Box::new("1.0 + 1")), Option::None);
  assert!(t.eval(None).is_err());

  let t = Timeset::new(0, 0, "t1", Some(Box::new("period")), Some(1.0 as f64));
  assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.0 as f64);

  let t = Timeset::new(0, 0, "t1", Some(Box::new("period + 0.25")), Some(1.0 as f64));
  assert_eq!(t.eval(Some(1.0 as f64)).unwrap(), 1.25 as f64);
}
