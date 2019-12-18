/// Endianness for PinCollections
#[derive(Debug)]
pub enum Endianness {
  LittleEndian, BigEndian
}

/// Model for a collection (or group) of pins
#[derive(Debug)]
pub struct PinCollection {
  pub ids: Vec<String>,
  pub endianness: Endianness,
  pub path: String,
}

impl PinCollection {
  pub fn new(path: &str, pin_ids: &Vec<String>, endianness: Option<Endianness>) -> PinCollection {
    PinCollection {
      path: path.to_string(),
      ids: pin_ids.iter().map( |p| String::from(p)).collect(),
      endianness: endianness.unwrap_or(Endianness::LittleEndian),
    }
  }

  pub fn len(&self) -> usize {
    self.ids.len()
  }
}
