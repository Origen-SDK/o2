use crate::error::Error;
use indexmap::map::IndexMap;
use std::collections::HashMap;
use std::convert::TryFrom;

/// List of supported pin actions.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
            _ => Err(Error::new(&format!(
                "Action {} is not available for pins!",
                s
            ))),
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
    
    pub fn as_tester_char(&self, data: u8) -> char {
        match self {
            PinActions::Drive => {
                match data {
                    0 => '0',
                    _ => '1',
                }
            },
            PinActions::Verify => {
                match data {
                    0 => 'L',
                    _ => 'H',
                }
            },
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
            _ => Err(Error::new(&format!(
                "Cannot derive PinActions enum from encoded character {}!",
                encoded_char
            ))),
        }
    }
}

/// Available Pin Roles
#[derive(Debug, Copy, Clone)]
pub enum PinRoles {
    Standard,
    Power,
    Ground,
    Virtual,
    Other,
}

/// Model for single pin.
#[derive(Debug, Clone)]
pub struct Pin {
    pub model_id: usize,
    pub id: usize,
    // Since pins will be added from the add_pin function of Pins,
    // just reuse that String instance instead of creating a new one.
    pub name: String,
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
    pub metadata: IndexMap<String, usize>,

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
            Err(Error::new(&format!(
                "Pin data must be either 0 or 1 - got {}",
                data
            )))
        }
    }

    pub fn reset(&mut self) {
        match self.reset_data {
            Some(d) => self.data = d as u8,
            None => {
                self.data = 0;
            }
        }
        match self.reset_action {
            Some(a) => self.action = a,
            None => self.action = PinActions::HighZ,
        }
    }

    pub fn add_metadata_id(&mut self, id_str: &str, id: usize) -> Result<(), Error> {
        if self.metadata.contains_key(id_str) {
            Err(Error::new(&format!(
                "Pin {} already has metadata {}! Use set_metadata to override its current value!",
                self.name, id_str,
            )))
        } else {
            self.metadata.insert(String::from(id_str), id);
            Ok(())
        }
    }

    pub fn get_metadata_id(&self, id_str: &str) -> Option<usize> {
        match self.metadata.get(id_str) {
            Some(id) => Some(*id),
            None => Option::None,
        }
    }

    pub fn new(
        model_id: usize,
        id: usize,
        name: String,
        reset_data: Option<u32>,
        reset_action: Option<PinActions>,
    ) -> Pin {
        let mut p = Pin {
            model_id: model_id,
            id: id,
            name: name,
            data: 0,
            action: PinActions::HighZ,
            reset_data: reset_data,
            reset_action: reset_action,
            aliases: Vec::new(),
            groups: HashMap::new(),
            role: PinRoles::Standard,
            metadata: IndexMap::new(),
        };
        p.reset();
        p
    }
}

impl Default for Pin {
    fn default() -> Pin {
        Self::new(
            0,
            0,
            String::from("default"),
            Option::None,
            Option::None,
        )
    }
}
