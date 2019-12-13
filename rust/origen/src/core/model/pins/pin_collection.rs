use super::PinContainer;

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
  pub fn new(pin_container: &mut PinContainer, pin_names: &Vec<&str>, endianness: Option<Endianness>) -> PinCollection {
    PinCollection {
      pins: pin_names.iter().map( |&p| String::from(p)).collect(),
      endianness: Endianness::LittleEndian
    }
  }

  fn posture(&mut self, pin_container: &mut PinContainer, data: i32) -> Result<String, String> {
    let mut d = data;
    for (i, n) in self.pins.iter_mut().enumerate() {
      let mut _p = pin_container.get_pin(n);
      if d & 1 == 1 {
        //_p.posture(true);
      } else {
        //_p.posture(false);
      }
      d = d >> 1;
    }
    Ok(String::from("Okay!"))
  }
  fn with_mask(&self, mask: i32) {}

  fn drive(&self) {
    println!("Driving pin values!");
  }
  fn assert(&self) {
    println!("Asserting pin values!");
  }
  //fn size(&self) {}
  //fn to_string(&self) {}
}
