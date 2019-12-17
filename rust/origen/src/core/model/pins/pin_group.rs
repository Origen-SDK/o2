//use super::pin::{PinActions};
//use crate::error::Error;
//use super::super::super::super::DUT;
use super::pin_collection::{Endianness};
//use crate::core::model::Model;


#[derive(Debug)]
pub struct PinGroup {
    pub name: String,
    pub path: String,
    pub pin_names: Vec<String>,
    pub endianness: Endianness,
}

impl PinGroup {
    pub fn new(name: String, path: String, pins: Vec<String>) -> PinGroup {
        return PinGroup {
            name: String::from(name),
            path: String::from(path),
            pin_names: pins,
            endianness: Endianness::LittleEndian,
        };
    }

    pub fn len(&self) -> usize {
        return self.pin_names.len();
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