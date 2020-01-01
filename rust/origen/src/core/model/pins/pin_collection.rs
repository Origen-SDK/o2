use crate::error::Error;
use super::super::pins::{Endianness};
use super::super::Model;
use super::pin::{PinActions};

/// Model for a collection (or group) of pins
#[derive(Debug, Clone)]
pub struct PinCollection {
  pub pin_names: Vec<String>,
  pub endianness: Endianness,
  pub path: String,
  pub mask: Option<usize>,
  pub model_id: usize,
}

impl PinCollection {
  pub fn new(model_id: usize, path: &str, pin_names: &Vec<String>, endianness: Option<Endianness>) -> PinCollection {
    PinCollection {
      path: path.to_string(),
      pin_names: match endianness {
        Some(e) => {
          match e {
            Endianness::LittleEndian => pin_names.iter().map( |p| String::from(p)).collect(),
            Endianness::BigEndian => pin_names.iter().rev().map( |p| String::from(p)).collect(),
          }
        },
        None => pin_names.iter().map( |p| String::from(p)).collect()
      },
      endianness: endianness.unwrap_or(Endianness::LittleEndian),
      mask: Option::None,
      model_id: model_id,
    }
  }

  pub fn len(&self) -> usize {
    self.pin_names.len()
  }

  pub fn slice_names(&self, start_idx: usize, stop_idx: usize, step_size: usize) -> Result<PinCollection, Error> {
    let mut sliced_names: Vec<String> = vec!();
    for i in (start_idx..=stop_idx).step_by(step_size) {
        if i >= self.pin_names.len() {
          return Err(Error::new(&format!("Index {} exceeds available pins in collection! (length: {})", i, self.pin_names.len())));
        }
        let p = self.pin_names[i].clone();
        sliced_names.push(p);
    }
    Ok(PinCollection::new(self.model_id, &self.path, &sliced_names, Option::Some(self.endianness)))
  }

  pub fn is_little_endian(&self) -> bool {
    match self.endianness {
      Endianness::LittleEndian => true,
      Endianness::BigEndian => false,
    }
  }

  pub fn is_big_endian(&self) -> bool {
    return !self.is_little_endian();
  }
}

impl Model {
  pub fn drive_pin_collection(&mut self, pin_collection: &mut PinCollection, data: Option<u32>) -> Result<(), Error> {
    self.set_pin_collection_actions(pin_collection, PinActions::Drive, data)
  }

  pub fn verify_pin_collection(&mut self, pin_collection: &mut PinCollection, data: Option<u32>) -> Result<(), Error> {
    self.set_pin_collection_actions(pin_collection, PinActions::Verify, data)
  }

  pub fn capture_pin_collection(&mut self, pin_collection: &mut PinCollection) -> Result<(), Error> {
    self.set_pin_collection_actions(pin_collection, PinActions::Capture, Option::None)
  }

  pub fn highz_pin_collection(&mut self, pin_collection: &mut PinCollection) -> Result<(), Error> {
    self.set_pin_collection_actions(pin_collection, PinActions::HighZ, Option::None)
  }

  pub fn set_pin_collection_actions(&mut self, collection: &mut PinCollection, action: PinActions, data: Option<u32>) -> Result<(), Error> {
    let pin_names = &collection.pin_names;
    let mask = collection.mask;
    collection.mask = Option::None;
    self.set_pin_actions(pin_names, action, data, mask)
  }

  pub fn get_pin_collection_data(&mut self, collection: &PinCollection) -> Result<u32, Error> {
    let pin_names = &collection.pin_names;
    Ok(self.get_pin_data(&pin_names))
  }

  pub fn get_pin_collection_reset_data(&mut self, collection: &PinCollection) -> u32 {
    let pin_names = &collection.pin_names;
    self.get_pin_reset_data(&pin_names)
  }

  pub fn get_pin_collection_reset_actions(&mut self, collection: &PinCollection) -> Result<String, Error> {
    let pin_names = &collection.pin_names;
    self.get_pin_reset_actions(&pin_names)
  }

  pub fn reset_pin_collection(&mut self,collection: &PinCollection) -> Result<(), Error> {
    let pin_names = &collection.pin_names;
    self.reset_pin_names(&pin_names)
  }

  pub fn set_pin_collection_data(&mut self, collection: &PinCollection, data: u32) -> Result<(), Error> {
    let pin_names = &collection.pin_names;
    self.set_pin_data(&pin_names, data, collection.mask)
  }

  pub fn set_pin_collection_nonsticky_mask(&mut self, pin_collection: &mut PinCollection, mask: usize) -> Result<(), Error> {
    pin_collection.mask = Some(mask);
    Ok(())
  }
}