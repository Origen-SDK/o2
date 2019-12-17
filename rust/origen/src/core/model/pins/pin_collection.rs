/// Endianness for PinCollections
#[derive(Debug)]
pub enum Endianness {
  LittleEndian, BigEndian
}

/// Model for a collection (or group) of pins
#[derive(Debug)]
pub struct PinCollection {
  pub pins: Vec<String>,
  pub endianness: Endianness,
}

impl PinCollection {
  pub fn new(pin_names: &Vec<&str>, endianness: Option<Endianness>) -> PinCollection {
    PinCollection {
      pins: pin_names.iter().map( |&p| String::from(p)).collect(),
      endianness: endianness.unwrap_or(Endianness::LittleEndian),
    }
  }

  pub fn drive(&self, _data: Option<usize>) {}
}
