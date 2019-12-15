use std::collections::HashMap;
use crate::error::Error;

/// List of supported pin actions.
#[derive(Debug)]
pub enum PinActions {
    Drive,
    Verify,
    Capture,
    HighZ,
}

impl PinActions {
    pub fn from_str(s: &str) -> Result<PinActions, Error> {
        match s {
            "Drive" => Ok(PinActions::Drive),
            "Verify" => Ok(PinActions::Verify),
            "Capture" => Ok(PinActions::Capture),
            "HighZ" => Ok(PinActions::HighZ),
            _ => Err(Error::new(&format!("Action {} is not available for pins!", s))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PinActions::Drive => "Drive",
            PinActions::Verify => "Verify",
            PinActions::Capture => "Capture",
            PinActions::HighZ => "HighZ",
        }
    }
}

/// The following types are allowed as metadata
#[derive(Debug)]
pub enum MetaAble {
    Text(String),
    Int(i32),
}

/// Available Pin Roles
#[derive(Debug)]
pub enum PinRoles {
    Standard,
    Power,
    Ground,
    Virtual,
    Other,
}

/// Model for single pin.
#[derive(Debug)]
pub struct Pin {
    // Since pins will be added from the add_pin function of Pins,
    // just reuse that String instance instead of creating a new one.
    pub name: String,
    pub path: String,

    /// The postured_state is the state the pin *would* be in, *if* it were driving/asserting
    /// a value. This allows for the bit values to be set without an action being applied as well.
    /// This can only be a 1 or 0, so use a bool to represent this.
    pub postured_state: bool,

    /// The pin's current action. If no action is desired, the pin will be HighZ.
    pub action: PinActions,

    /// The pin's initial action and state. This will be applied during creation and whenever the
    /// 'reset' function is called.
    pub initial: (PinActions, bool),
    //pub size: i32,
    ///--- Meta Data ---///
    /// Any aliases this Pin has.
    pub aliases: Vec<String>,
    pub role: PinRoles,
    pub meta: HashMap<String, MetaAble>,
    pub data: u8,

    // Taking the speed over size here: this'll allow for quick lookups and indexing from pins into the pin group, but will
    // require a bit of extra storage. Since that storage is only a reference and uint, it should be small and well worth the
    // lookup boost.
    pub memberships: HashMap<String, i32>,
}

impl Pin {
    pub fn posture(&mut self, data: bool) {
        self.postured_state = data;
    }
    pub fn drive(&mut self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }
        self.action = PinActions::Drive;
        Ok(())
    }

    pub fn verify(&mut self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }
        self.action = PinActions::Verify;
        Ok(())
    }

    pub fn capture(&mut self) -> Result<(), Error> {
        self.action = PinActions::Capture;
        Ok(())
    }

    pub fn highz(&mut self) -> Result<(), Error> {
        self.action = PinActions::HighZ;
        Ok(())
    }

    pub fn new(name: String, path: String) -> Pin {
        return Pin {
            name: name,
            action: PinActions::HighZ,
            postured_state: false,
            initial: (PinActions::HighZ, false),
            aliases: Vec::new(),
            memberships: HashMap::new(),
            role: PinRoles::Standard,
            meta: HashMap::new(),
            data: 0,
            path: path,
        };
    }

    pub fn set_data(&mut self, data: u8) -> Result<(), Error> {
        if data == 0 || data == 1 {
            self.data = data;
            Ok(())
        } else {
            Err(Error::new(&format!("Pin data must be either 0 or 1 - got {}", data)))
        }
    }
}

impl Default for Pin {
    fn default() -> Pin {
        return Pin {
            name: String::from("default"),
            action: PinActions::HighZ,
            postured_state: false,
            initial: (PinActions::HighZ, false),
            aliases: Vec::new(),
            memberships: HashMap::new(),
            role: PinRoles::Standard,
            meta: HashMap::new(),
            data: 0,
            path: String::from(""),
        };
    }
}

#[derive(Debug)]
pub struct PinGroup {
    name: String,
    path: String,
    pin_names: Vec<String>
}

impl PinGroup {
    pub fn new(name: String, path: String, pins: Vec<String>) -> PinGroup {
        return PinGroup {
            name: String::from(name),
            path: String::from(path),
            pin_names: pins,
        };
    }

    pub fn len(&self) -> usize {
        return self.pin_names.len();
    }
}
