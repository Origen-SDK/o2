use crate::error::Error;

/// Endianness for PinCollections
#[derive(Debug, Copy, Clone)]
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

  pub fn slice_ids(&self, start_idx: usize, stop_idx: usize, step_size: usize) -> Result<PinCollection, Error> {
    let mut sliced_ids: Vec<String> = vec!();
    for i in (start_idx..=stop_idx).step_by(step_size) {
        if i >= self.ids.len() {
          return Err(Error::new(&format!("Index {} exceeds available pins in collection! (length: {})", i, self.ids.len())));
        }
        let p = self.ids[i].clone();
        sliced_ids.push(p);
    }
    Ok(PinCollection::new(&self.path, &sliced_ids, Option::Some(self.endianness)))
  }
}
