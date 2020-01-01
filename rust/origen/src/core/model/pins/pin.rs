use std::collections::HashMap;
use std::convert::TryFrom;
use crate::error::Error;

/// List of supported pin actions.
#[derive(Debug, Copy, Clone)]
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
            "D" => Ok(PinActions::Drive),
            "V" => Ok(PinActions::Verify),
            "C" => Ok(PinActions::Capture),
            "Z" => Ok(PinActions::HighZ),
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

    pub fn as_char(&self) -> char {
        match self {
            PinActions::Drive => 'D',
            PinActions::Verify => 'V',
            PinActions::Capture => 'C',
            PinActions::HighZ => 'Z',
        }
    }
}

impl TryFrom<u8> for PinActions {
    type Error = crate::error::Error;
    fn try_from(encoded_char: u8) -> Result<Self, Self::Error> {
        let c = char::from(encoded_char);
        match c {
            'd' => Ok(PinActions::Drive),
            'D' => Ok(PinActions::Drive),
            'v' => Ok(PinActions::Verify),
            'V' => Ok(PinActions::Verify),
            'c' => Ok(PinActions::Capture),
            'C' => Ok(PinActions::Capture),
            'z' => Ok(PinActions::HighZ),
            'Z' => Ok(PinActions::HighZ),
            _ => Err(Error::new(&format!("Cannot derive PinActions enum from encoded character {}!", encoded_char))),
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
    pub data: u8,

    /// The pin's current action. If no action is desired, the pin will be HighZ.
    pub action: PinActions,

    /// The pin's initial action and state. This will be applied during creation and whenever the
    /// 'reset' function is called.
    pub reset_action: Option<PinActions>,
    pub reset_data: Option<u32>,

    ///--- Meta Data ---///
    /// Any aliases this Pin has.
    pub aliases: Vec<String>,
    pub role: PinRoles,
    pub meta: HashMap<String, MetaAble>,

    // Taking the speed over size here: this'll allow for quick lookups and indexing from pins into the pin group, but will
    // require a bit of extra storage. Since that storage is only a reference and uint, it should be small and well worth the
    // lookup boost.
    pub groups: HashMap<String, usize>,
}

impl Pin {
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

    pub fn set_data(&mut self, data: u8) -> Result<(), Error> {
        if data == 0 || data == 1 {
            self.data = data;
            Ok(())
        } else {
            Err(Error::new(&format!("Pin data must be either 0 or 1 - got {}", data)))
        }
    }

    pub fn reset(&mut self) {
        match self.reset_data {
            Some(d) => { self.data = d as u8 },
            None => { self.data = 0; },
        }
        match self.reset_action {
            Some(a) => { self.action = a },
            None => { self.action = PinActions::HighZ },
        }
    }

    pub fn new(name: String, path: String, reset_data: Option<u32>, reset_action: Option<PinActions>) -> Pin {
        let mut p = Pin {
            name: name,
            path: path,
            data: 0,
            action: PinActions::HighZ,
            reset_data: reset_data,
            reset_action: reset_action,
            aliases: Vec::new(),
            groups: HashMap::new(),
            role: PinRoles::Standard,
            meta: HashMap::new(),
        };
        p.reset();
        p
    }
}

impl Default for Pin {
    fn default() -> Pin {
        Self::new(String::from("default"), String::from(""), Option::None, Option::None)
    }
}
