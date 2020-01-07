pub mod timeset;

use super::Model;
use timeset::Timeset;
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
  pub fn add_timeset(&mut self, _model_id: usize, instance_id: usize, name: &str, period: Option<Box<dyn std::string::ToString>>, default_period: Option<f64>) -> Result<Timeset, Error> {
    let t = Timeset::new(name, period, default_period);
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

  // pub fn _get_mut_timeset(&mut self, name: &str) -> Result<&mut Timeset, Error> {
  //   match self.get_mut_timeset(name) {
  //     Some(t) => Ok(t),
  //     None => no_pin_group_error!()
  //   }
  // }
}
