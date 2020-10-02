use crate::error::Error;
use indexmap::map::IndexMap;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::RwLock;
use crate::standards::actions::*;

pub trait ResolvePinActions {
    fn pin_action_resolver(&self, target: String) -> &Resolver;
    fn mut_pin_action_resolver(&mut self, target: String) -> &mut Resolver;

    fn resolve_pin_action(&self, target: String, action: &PinAction) -> Option<String> {
        self.pin_action_resolver(target).resolve(action)
    }

    fn _resolve_pin_action(&self, target: String, action: &PinAction) -> Result<String, Error> {
        match self.resolve_pin_action(target, action) {
            Some(a) => Ok(a),
            None => Err(Error::new(&format!(
                "No resolution provided for pin action {:?}",
                &action
            )))
        }
    }

    fn resolve_pin_actions(&self, target: String, actions: &Vec<PinAction>) -> Vec<Option<String>> {
        self.pin_action_resolver(target).resolve_all(&actions)
    }

    fn update_mapping(&mut self, target: String, action: PinAction, new_resolution: String) {
        self.mut_pin_action_resolver(target).update_mapping(action, new_resolution)
    }
}

#[derive(Debug, Clone)]
pub struct Resolver {
    resolution_map: IndexMap::<PinAction, String>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            resolution_map: IndexMap::new()
        }
    }

    pub fn resolve(&self, action: &PinAction) -> Option<String> {
        if let Some(r) = self.resolution_map.get(action) {
            Some(r.clone())
        } else {
            None
        }
    }

    pub fn update_mapping(&mut self, action: PinAction, new_resolution: String) {
        self.resolution_map.insert(action, new_resolution);
    }

    pub fn resolve_all(&self, actions: &Vec<PinAction>) -> Vec<Option<String>> {
        actions.iter().map(|a| self.resolve(a)).collect()
    }

    pub fn mapping(&self) -> &IndexMap<PinAction, String> {
        &self.resolution_map
    }
}

/// Single 'action' applicable to a pin
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PinAction {
    action: String,
}

impl PinAction {
    pub fn new(action: &str) -> Self {
        Self {
            action: action.to_string(),
            // metadata: None,
        }
    }
    // pub fn standard_actions() -> Vec<String> {}

    pub fn to_string(&self) -> String {
        self.action.to_string()
    }

    pub fn drive_high() -> Self {
        Self::new(DRIVE_HIGH)
    }

    pub fn drive_low() -> Self {
        Self::new(DRIVE_LOW)
    }

    pub fn verify_high() -> Self {
        Self::new(VERIFY_HIGH)
    }
    pub fn verify_low() -> Self {
        Self::new(VERIFY_LOW)
    }

    pub fn capture() -> Self {
        Self::new(CAPTURE)
    }

    pub fn highz() -> Self {
        Self::new(HIGHZ)
    }
}

// /// List of supported pin actions.
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
// pub enum PinActions {
//     Drive,
//     DriveHigh,
//     DriveLow,
//     Verify,
//     VerifyHigh,
//     VerifyLow,
//     Capture,
//     HighZ,
//     Other(String)
// }

// // impl std::fmt::Display for PinActions {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         write!(f, "{}", self.to_string().unwrap())
// //     }
// // }

// impl PinActions {

//     pub fn standard_actions() -> Vec<PinActions> {
//         vec!(Self::Drive, Self::DriveHigh, Self::DriveLow,
//         Self::Verify, Self::VerifyHigh, Self::VerifyLow,
//         Self::Capture, Self::HighZ)
//     }

//     /// Converts a str representing symbols of arbitrary size into a vector of PinAction::Other types.
//     /// Complains if the symbols are not divisible by the symbol size.
//     pub fn from_symbol_str(symbols: &str) -> Result<Vec<PinActions>, Error> {
//         let mut retn: Vec<Self> = vec![];
//         let mut sym = String::new();
//         let mut in_sym = false;
//         for s in symbols.chars() {
//             let mut b = [0; 4];
//             let c = &*s.encode_utf8(&mut b);
//             if in_sym {
//                 if c == "|" {
//                     retn.push(Self::Other(sym.clone()));
//                     sym.clear();
//                     in_sym = false;
//                 } else {
//                     sym.push_str(c);
//                 }
//             } else if c == "|" {
//                 in_sym = true;
//             } else {
//                 let c_ = Self::from(c);
//                 match c_ {
//                     Self::Other(_) => return Err(Error::new(&format!(
//                         "Cannot derive PinActions enum from encoded character {}!",
//                         c
//                     ))),
//                     _ => retn.push(c_),
//                 }
//             }
//         }
//         if in_sym {
//             // Current sym wasn't closed. Badly formatted input
//             return Err(Error::new(&format!(
//                 "Badly formatted PinActions string: Open user-defined symbol without closing delimiter. Current symbol: {}",
//                 &sym
//             )));
//         }
//         Ok(retn)
//     }

//     pub fn push_to_string(&self, current: &mut String) -> Result<(), Error> {
//         match self {
//             Self::Other(s) => current.push_str(&format!("|{}|", s)),
//             _ => current.push(self.as_char())
//         }
//         Ok(())
//     }

//     // pub fn long_name(&self) -> String {
//     //     match self {
//     //         PinActions::Drive => "Drive".into(),
//     //         PinActions::Verify => "Verify".into(),
//     //         Self::DriveHigh => "DriveHigh".into(),
//     //         Self::DriveLow => "DriveLow".into(),
//     //         Self::VerifyHigh => "VerifyHigh".into(),
//     //         Self::VerifyLow => "VerifyLow".into(),
//     //         PinActions::Capture => "Capture".into(),
//     //         PinActions::HighZ => "HighZ".into(),
//     //         PinActions::Other(sym) => format!("Other({})", sym)
//     //     }
//     // }

//     pub fn to_action_string(actions: &Vec<PinActions>) -> Result<String, Error> {
//         let mut retn = "".to_string();
//         for action in actions.iter().rev() {
//             action.push_to_string(&mut retn)?;
//         }
//         return Ok(retn)
//     }

//     pub fn to_string(&self) -> Result<String, Error> {
//         Self::to_action_string(&vec!(self.clone()))
//     }

//     pub fn as_char(&self) -> char {
//         match self {
//             PinActions::Drive => 'D',
//             Self::DriveHigh => '1',
//             Self::DriveLow => '0',
//             PinActions::Verify => 'V',
//             Self::VerifyHigh => 'H',
//             Self::VerifyLow => 'L',
//             PinActions::Capture => 'C',
//             PinActions::HighZ => 'Z',
//             PinActions::Other(_) => '_',
//         }
//     }

//     pub fn as_sym(&self) -> String {
//         match self {
//             PinActions::Other(sym) => sym.clone(),
//             _ => self.as_char().to_string()
//         }
//     }

//     pub fn apply_state(&self, state: u8) -> Self {
//         match self {
//             Self::Drive => {
//                 if state == 0 {
//                     Self::DriveLow
//                 } else {
//                     Self::DriveHigh
//                 }
//             },
//             Self::Verify => {
//                 if state == 0 {
//                     Self::VerifyLow
//                 } else {
//                     Self::VerifyHigh
//                 }
//             },
//             _ => self.clone()
//         }
//     }

//     pub fn is_standard(&self) -> bool {
//         match self {
//             Self::Other(_) => false,
//             _ => true
//         }
//     }

//     /// Similar to "from", but accepts a custom string whether or not its delimited.
//     ///    from_delimiter_optional('a') => Other('a')
//     ///    from_delimiter_optional('|a|') => Other('a')
//     /// Delimiters are needed to override standard actions, however:
//     ///    from_delimiter_optional('1') => DriveHigh
//     ///    from_delimiter_optional('|1|') => Other('1')
//     /// The input must also be a single action only.
//     pub fn from_delimiter_optional(action: &str) -> Result<Self, Error> {
//         if action.is_empty() {
//             return Err(Error::new(&format!(
//                 "Improperly formatted delimited-optional action. Action cannot be empty"
//             )));
//         }

//         let matches: Vec<_> = action.match_indices('|').collect();
//         if matches.len() == 0 {
//             // No '|' were found, so just the action remains - if it matches a standard action,
//             // use that, otherwise generate a custom action
//             Ok(Self::from(action))
//         } else if matches.len() == 2 {
//             // Since this keeps the delimiters, there are exactly two '|' and one 'other thing'
//             // in the string.
//             // The order could be screwed up, however. '||hi' will give you [|, |, hi],
//             // so verify that the first and last indices are '|' to check for properly formatted input
//             if let Some(temp) = action.strip_prefix("|") {
//                 if let Some(temp2) = temp.strip_suffix("|") {
//                     return Ok(Self::Other(temp2.to_string()));
//                 }
//             }
//             Err(Error::new(&format!(
//                 "Improperly formatted delimited-optional action. Cannot convert from '{}'",
//                 action
//             )))
//         } else {
//             // Unexpected input. Complain.
//             Err(Error::new(&format!(
//                 "Expected a single delimited-optional action. Cannot convert from '{}'",
//                 action
//             )))
//         }
//     }
// }

// impl TryFrom<u8> for PinActions {
//     type Error = crate::error::Error;
//     fn try_from(encoded_char: u8) -> Result<Self, Self::Error> {
//         let c = char::from(encoded_char);
//         match c {
//             'd' => Ok(PinActions::Drive),
//             'D' => Ok(PinActions::Drive),
//             'v' => Ok(PinActions::Verify),
//             'V' => Ok(PinActions::Verify),
//             '1' => Ok(Self::DriveHigh),
//             '0' => Ok(Self::DriveLow),
//             'H' => Ok(Self::VerifyHigh),
//             'h' => Ok(Self::VerifyHigh),
//             'L' => Ok(Self::VerifyLow),
//             'l' => Ok(Self::VerifyLow),
//             'c' => Ok(PinActions::Capture),
//             'C' => Ok(PinActions::Capture),
//             'z' => Ok(PinActions::HighZ),
//             'Z' => Ok(PinActions::HighZ),
//             _ => Err(Error::new(&format!(
//                 "Cannot derive PinActions enum from encoded character {}!",
//                 encoded_char
//             ))),
//         }
//     }
// }

// impl From<&str> for PinActions {
//     fn from(s: &str) -> Self {
//         if s.len() == 1 {
//             match Self::try_from(s.chars().next().unwrap() as u8) {
//                 Ok(p) => p,
//                 Err(_) => Self::Other(s.to_string())
//             }
//         } else {
//             Self::Other(s.to_string())
//         }
//     }
// }

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
#[derive(Debug)]
pub struct Pin {
    pub model_id: usize,
    pub id: usize,
    // Since pins will be added from the add_pin function of Pins,
    // just reuse that String instance instead of creating a new one.
    pub name: String,
    pub data: RwLock<u8>,

    // /// The pin's current action. If no action is desired, the pin will be HighZ.
    // pub action: PinActions,
    pub action: RwLock<PinAction>,

    /// The pin's initial action and state. This will be applied during creation and whenever the
    /// 'reset' function is called.
    pub reset_action: Option<PinAction>,
    // pub reset_data: Option<u32>,

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
    pub fn drive(&self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }

        let mut action = self.action.write().unwrap();
        if *self.data.read().unwrap() == 0 {
            *action = PinAction::drive_low();
        } else {
            *action = PinAction::drive_high();
        }
        Ok(())
    }

    pub fn verify(&self, data: Option<u8>) -> Result<(), Error> {
        if let Some(d) = data {
            self.set_data(d)?;
        }

        let mut action = self.action.write().unwrap();
        if *self.data.read().unwrap() == 0 {
            *action = PinAction::verify_low();
        } else {
            *action = PinAction::verify_high();
        }
        Ok(())
    }

    pub fn capture(&self) -> Result<(), Error> {
        let mut action = self.action.write().unwrap();
        *action = PinAction::capture();
        Ok(())
    }

    pub fn highz(&self) -> Result<(), Error> {
        let mut action = self.action.write().unwrap();
        *action = PinAction::highz();
        Ok(())
    }

    pub fn set_data(&self, data: u8) -> Result<(), Error> {
        if data == 0 || data == 1 {
            let mut pin_data = self.data.write().unwrap();
            *pin_data = data;
            Ok(())
        } else {
            Err(Error::new(&format!(
                "Pin data must be either 0 or 1 - got {}",
                data
            )))
        }
    }

    pub fn reset(&self) {
        let mut action = self.action.write().unwrap();
        // {
        //     let mut data = self.data.write().unwrap();
        //     match self.reset_data {
        //         Some(d) => *data = d as u8,
        //         None => {
        //             *data = 0;
        //         }
        //     }
        // }
        match self.reset_action.as_ref() {
            // Some(a) => *action = a.apply_state(*self.data.read().unwrap()),
            Some(a) => *action = a.clone(),
            None => *action = PinAction::highz(),
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
        reset_action: Option<PinAction>,
    ) -> Pin {
        let p = Pin {
            model_id: model_id,
            id: id,
            name: name,
            data: RwLock::new(0),
            action: RwLock::new(PinAction::highz()),
            // reset_data: reset_data,
            // reset_action: {
            //     if let Some(a) = reset_action {
            //         Some(a.apply_state(reset_data.unwrap_or(0) as u8))
            //     } else {
            //         None
            //     }
            // },
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

impl std::clone::Clone for Pin {
    fn clone(&self) -> Self {
        println!("Cloning pin... where is this used?");
        Self::new(0, 0, String::from("default"), Option::None, Option::None)
    }
}
