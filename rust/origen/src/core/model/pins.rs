pub mod pin;
use crate::error::Error;

use std::collections::HashMap;

/// Structure to contain the added pins.
pub struct PinContainer {
    pub pins: HashMap<String, pin::Pin>,
    pub aliases: HashMap<String, String>,
}

impl PinContainer {
    pub fn new() -> PinContainer {
        PinContainer {
            pins: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn add_pin(&mut self, name: String) {
        let n = name;
        let p = pin::Pin::new(n.clone());
        self.pins.insert(n, p);
    }

    pub fn get_pin(&mut self, pin: &str) -> Result<&mut pin::Pin, Error> {
        if let Some(p) = self.pins.get_mut(pin) {
          Ok(p)
        } else {
          Err(Error::new(&format!("No pin {} available!", pin)))
        }
    }
}
