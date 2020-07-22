use crate::error::Error;
use indexmap::map::IndexMap;
use std::collections::HashMap;
use std::convert::TryFrom;

pub trait ResolvePinActions {
    fn pin_action_resolver(&self) -> &Resolver;
    fn mut_pin_action_resolver(&mut self) -> &mut Resolver;

    fn resolve_pin_action(&self, action: &PinActions) -> Option<String> {
        self.pin_action_resolver().resolve(action)
    }

    fn _resolve_pin_action(&self, action: &PinActions) -> Result<String, Error> {
        match self.resolve_pin_action(action) {
            Some(a) => Ok(a),
            None => Err(Error::new(&format!(
                "No resolution provided for pin action {}",
                &action.long_name()
            )))
        }
    }

    fn resolve_pin_actions(&self, actions: &Vec<PinActions>) -> Vec<Option<String>> {
        self.pin_action_resolver().resolve_all(&actions)
    }

    fn update_mapping(&mut self, action: PinActions, new_resolution: String) {
        self.mut_pin_action_resolver().update_mapping(action, new_resolution)
    }
}

#[derive(Debug, Clone)]
pub struct Resolver {
    resolution_map: IndexMap::<PinActions, String>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            resolution_map: IndexMap::new()
        }
    }

    pub fn resolve(&self, action: &PinActions) -> Option<String> {
        if let Some(r) = self.resolution_map.get(action) {
            Some(r.clone())
        } else {
            None
        }
    }

    pub fn update_mapping(&mut self, action: PinActions, new_resolution: String) {
        self.resolution_map.insert(action, new_resolution);
    }

    pub fn resolve_all(&self, actions: &Vec<PinActions>) -> Vec<Option<String>> {
        actions.iter().map(|a| self.resolve(a)).collect()
    }

    pub fn mapping(&self) -> &IndexMap<PinActions, String> {
        &self.resolution_map
    }
}

/// List of supported pin actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum PinActions {
    Drive,
    DriveHigh,
    DriveLow,
    Verify,
    VerifyHigh,
    VerifyLow,
    Capture,
    HighZ,
    Other(String)
}

impl PinActions {

    pub fn standard_actions() -> Vec<PinActions> {
        vec!(Self::Drive, Self::DriveHigh, Self::DriveLow,
        Self::Verify, Self::VerifyHigh, Self::VerifyLow,
        Self::Capture, Self::HighZ)
    }

    /// Converts a str representing symbols of arbitrary size into a vector of PinAction::Other types.
    /// Complains if the symbols are not divisible by the symbol size.
    pub fn from_symbol_str(symbols: &str) -> Result<Vec<PinActions>, Error> {
        let mut retn: Vec<Self> = vec![];
        let mut sym = String::new();
        let mut in_sym = false;
        for s in symbols.chars() {
            let mut b = [0; 4];
            let c = &*s.encode_utf8(&mut b);
            if in_sym {
                if c == "|" {
                    retn.push(Self::Other(sym.clone()));
                    sym.clear();
                    in_sym = false;
                } else {
                    sym.push_str(c);
                }
            } else if c == "|" {
                in_sym = true;
            } else {
                let c_ = Self::from(c);
                match c_ {
                    Self::Other(_) => return Err(Error::new(&format!(
                        "Cannot derive PinActions enum from encoded character {}!",
                        c
                    ))),
                    _ => retn.push(c_),
                }
            }
        }
        if in_sym {
            // Current sym wasn't closed. Badly formatted input
            return Err(Error::new(&format!(
                "Badly formatted PinActions string: Open user-defined symbol without closing delimiter. Current symbol: {}",
                &sym
            )));
        }
        Ok(retn)
    }

    pub fn push_to_string(&self, current: &mut String) -> Result<(), Error> {
        match self {
            Self::Other(s) => current.push_str(&format!("|{}|", s)),
            _ => current.push(self.as_char())
        }
        Ok(())
    }

    pub fn long_name(&self) -> String {
        match self {
            PinActions::Drive => "Drive".into(),
            PinActions::Verify => "Verify".into(),
            Self::DriveHigh => "DriveHigh".into(),
            Self::DriveLow => "DriveLow".into(),
            Self::VerifyHigh => "VerifyHigh".into(),
            Self::VerifyLow => "VerifyLow".into(),
            PinActions::Capture => "Capture".into(),
            PinActions::HighZ => "HighZ".into(),
            PinActions::Other(sym) => format!("Other({})", sym)
        }
    }

    pub fn to_action_string(actions: &Vec<PinActions>) -> String {
        let mut retn = "".to_string();
        for action in actions.iter().rev() {
            action.push_to_string(&mut retn);
        }
        return retn
    }

    pub fn to_string(&self) -> String {
        Self::to_action_string(&vec!(self.clone()))
    }

    pub fn as_char(&self) -> char {
        match self {
            PinActions::Drive => 'D',
            Self::DriveHigh => '1',
            Self::DriveLow => '0',
            PinActions::Verify => 'V',
            Self::VerifyHigh => 'H',
            Self::VerifyLow => 'L',
            PinActions::Capture => 'C',
            PinActions::HighZ => 'Z',
            PinActions::Other(_) => '_',
        }
    }

    pub fn apply_state(&self, state: u8) -> Self {
        match self {
            Self::Drive => {
                if state == 0 {
                    Self::DriveLow
                } else {
                    Self::DriveHigh
                }
            },
            Self::Verify => {
                if state == 0 {
                    Self::VerifyLow
                } else {
                    Self::VerifyHigh
                }
            },
            _ => self.clone()
        }
    }

    pub fn is_standard(&self) -> bool {
        match self {
            Self::Other(_) => false,
            _ => true
        }
    }

    /// Similar to "from", but accepts a custom string whether or not its delimited.
    ///    from_delimiter_optional('a') => Other('a')
    ///    from_delimiter_optional('|a|') => Other('a')
    /// Delimiters are needed to override standard actions, however:
    ///    from_delimiter_optional('1') => DriveHigh
    ///    from_delimiter_optional('|1|') => Other('1')
    /// The input must also be a single action only.
    pub fn from_delimiter_optional(action: &str) -> Result<Self, Error> {
        if action.is_empty() {
            return Err(Error::new(&format!(
                "Improperly formatted delimited-optional action. Action cannot be empty"
            )));
        }

        let matches: Vec<_> = action.match_indices('|').collect();
        if matches.len() == 0 {
            // No '|' were found, so just the action remains - if it matches a standard action,
            // use that, otherwise generate a custom action
            Ok(Self::from(action))
        } else if matches.len() == 2 {
            // Since this keeps the delimiters, there are exactly two '|' and one 'other thing'
            // in the string.
            // The order could be screwed up, however. '||hi' will give you [|, |, hi],
            // so verify that the first and last indices are '|' to check for properly formatted input
            if let Some(temp) = action.strip_prefix("|") {
                if let Some(temp2) = temp.strip_suffix("|") {
                    return Ok(Self::Other(temp2.to_string()));
                }
            }
            Err(Error::new(&format!(
                "Improperly formatted delimited-optional action. Cannot convert from '{}'",
                action
            )))
        } else {
            // Unexpected input. Complain.
            Err(Error::new(&format!(
                "Expected a single delimited-optional action. Cannot convert from '{}'",
                action
            )))
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
            '1' => Ok(Self::DriveHigh),
            '0' => Ok(Self::DriveLow),
            'H' => Ok(Self::VerifyHigh),
            'h' => Ok(Self::VerifyHigh),
            'L' => Ok(Self::VerifyLow),
            'l' => Ok(Self::VerifyLow),
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

impl From<&str> for PinActions {
    fn from(s: &str) -> Self {
        if s.len() == 1 {
            match Self::try_from(s.chars().next().unwrap() as u8) {
                Ok(p) => p,
                Err(_) => Self::Other(s.to_string())
            }
        } else {
            Self::Other(s.to_string())
        }
    }
}

// pub struct PinIdentifier {
//     pub model_id: usize,
//     pub name: usize
// }

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

    //pub overlaying: bool,
}

impl Pin {
    pub fn drive(&mut self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }
        if self.data == 0 {
            self.action = PinActions::DriveLow;
        } else {
            self.action = PinActions::DriveHigh;
        }
        Ok(())
    }

    pub fn verify(&mut self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }
        if self.data == 0 {
            self.action = PinActions::VerifyLow;
        } else {
            self.action = PinActions::VerifyHigh;
        }
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
        match self.reset_action.as_ref() {
            Some(a) => self.action = a.apply_state(self.data),
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
            reset_action: {
                if let Some(a) = reset_action {
                    Some(a.apply_state(reset_data.unwrap_or(0) as u8))
                } else {
                    None
                }
            },
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
        Self::new(0, 0, String::from("default"), Option::None, Option::None)
    }
}
