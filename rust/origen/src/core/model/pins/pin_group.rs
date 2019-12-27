//use super::pin::{PinActions};
//use crate::error::Error;
//use super::super::super::super::DUT;
use super::pin_collection::{Endianness};
//use crate::core::model::Model;


// We'll maintain both the pin_ids which the group was built with, but we'll also maintain the list
// of physical IDs. Even though we can resolve this later, most operations wil 
#[derive(Debug, Clone)]
pub struct PinGroup {
    pub id: String,
    pub path: String,
    pub pin_ids: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
}

impl PinGroup {
    pub fn new(id: String, path: String, pins: Vec<String>) -> PinGroup {
        return PinGroup {
            id: String::from(id),
            path: String::from(path),
            pin_ids: pins,
            endianness: Endianness::LittleEndian,
            mask: Option::None,
        };
    }

    pub fn len(&self) -> usize {
        return self.pin_ids.len();
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