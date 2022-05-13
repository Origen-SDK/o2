use crate::standards::actions::*;
use crate::Result;
use indexmap::map::IndexMap;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::sync::RwLock;

pub trait ResolvePinActions {
    fn pin_action_resolver(&self, target: String) -> &Resolver;
    fn mut_pin_action_resolver(&mut self, target: String) -> &mut Resolver;

    fn resolve_pin_action(&self, target: String, action: &PinAction) -> Option<String> {
        self.pin_action_resolver(target).resolve(action)
    }

    fn _resolve_pin_action(&self, target: String, action: &PinAction) -> Result<String> {
        match self.resolve_pin_action(target, action) {
            Some(a) => Ok(a),
            None => Err(error!(
                "No resolution provided for pin action {:?}",
                &action
            )),
        }
    }

    fn resolve_pin_actions(&self, target: String, actions: &Vec<PinAction>) -> Vec<Option<String>> {
        self.pin_action_resolver(target).resolve_all(&actions)
    }

    fn update_mapping(&mut self, target: String, action: PinAction, new_resolution: String) {
        self.mut_pin_action_resolver(target)
            .update_mapping(action, new_resolution)
    }
}

#[derive(Debug, Clone)]
pub struct Resolver {
    resolution_map: IndexMap<PinAction, String>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            resolution_map: IndexMap::new(),
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
#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct PinAction {
    pub action: String,
}

impl PartialEq for PinAction {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl PinAction {
    pub fn new<T: AsRef<str> + std::fmt::Display>(action: T) -> Self {
        Self {
            action: action.to_string(),
            // metadata: None,
        }
    }

    pub fn checked_new<T: AsRef<str> + std::fmt::Display>(action: T) -> Result<Self> {
        let a = Self::from_action_str(&action.to_string())?;
        if a.len() > 1 {
            bail!(
                "Expected Single PinAction but input {} would resolve to multiple actions",
                action
            )
        } else {
            Ok(Self::new(a.first().unwrap().to_string()))
        }
    }

    pub fn from_delimiter_optional<T: AsRef<str> + std::fmt::Display>(action: T) -> Result<Self> {
        let mut s = action.to_string();
        if !s.starts_with("|") {
            s = format!("|{}|", s);
        }
        Self::checked_new(s)
    }

    pub fn standard_actions() -> std::collections::HashMap<String, String> {
        standard_actions()
    }

    pub fn is_standard(&self) -> bool {
        STANDARD_ACTIONS.contains_key(&self.action)
    }

    pub fn to_string(&self) -> String {
        self.action.to_string()
    }

    pub fn from_action_str(actions: &str) -> Result<Vec<Self>> {
        let mut retn: Vec<Self> = vec![];
        let mut sym = String::new();
        let mut in_sym = false;

        for a in actions.chars() {
            let mut b = [0; 4];
            let c = &*a.encode_utf8(&mut b);
            if in_sym {
                if c == "|" {
                    retn.push(Self::new(&sym));
                    sym.clear();
                    in_sym = false;
                } else {
                    sym.push_str(c);
                }
            } else if c == "|" {
                in_sym = true;
            } else {
                retn.push(Self::new(c));
            }
        }
        if in_sym {
            // Current sym wasn't closed. Badly formatted input
            bail!(
                "Badly formatted PinAction string: Open user-defined symbol without closing delimiter. Current symbol: {}",
                &sym
            );
        }
        Ok(retn)
    }

    pub fn push_to_string(&self, current: &mut String) -> Result<()> {
        if self.action.len() > 1 {
            current.push_str(&format!("|{}|", self.action));
        } else {
            current.push_str(&self.action);
        }
        Ok(())
    }

    pub fn to_action_string(actions: &Vec<Self>) -> Result<String> {
        let mut retn = "".to_string();
        for action in actions.iter().rev() {
            action.push_to_string(&mut retn)?;
        }
        return Ok(retn);
    }

    pub fn to_bool(&self) -> Result<bool> {
        if self.action == DRIVE_HIGH || self.action == VERIFY_HIGH {
            Ok(true)
        } else if self.action == DRIVE_LOW || self.action == VERIFY_LOW {
            Ok(false)
        } else {
            bail!("Cannot interpret action {} as a logic 1 or 0", self.action)
        }
    }

    pub fn to_bool_unchecked(&self) -> bool {
        self.action == DRIVE_HIGH || self.action == VERIFY_HIGH
    }

    pub fn to_logic(&self) -> Result<usize> {
        Ok(self.to_bool()?.into())
    }

    pub fn to_logic_unchecked(&self) -> usize {
        self.to_bool_unchecked().into()
    }

    // pub fn is_driving(&self) -> bool {
    //     self.action == DRIVE_HIGH || self.action == DRIVE_LOW
    // }

    // pub fn is_writing(&self) -> bool {
    //     self.is_driving()
    // }

    // pub fn is_verifying(&self) -> bool {
    //     self.action == VERIFY_HIGH || self.action == VERIFY_LOW
    // }

    pub fn to_data(actions: Vec<PinAction>) -> Result<BigUint> {
        let mut retn = BigUint::new(vec![0]);
        for (i, a) in actions.iter().rev().enumerate() {
            match a.to_bool() {
                Ok(b) => {
                    retn <<= 1;
                    if b {
                        retn = retn + (1 as u8);
                    }
                },
                Err(_e) => bail!(
                    "Cannot convert actions to data as action {} at index {} cannot be interpreted as a logic 1 or 0",
                    &a.to_string(),
                    (actions.len() - i - 1) // Re-reverse the indices to they make sense to the user
                )
            }
        }
        Ok(retn)
    }

    pub fn to_data_unchecked(actions: Vec<PinAction>) -> BigUint {
        let mut retn = BigUint::new(vec![0]);
        for a in actions.iter().rev() {
            retn <<= 1;
            if a.to_bool_unchecked() {
                retn = retn + (1 as u8);
            }
        }
        retn
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

    pub reset_action: Option<PinAction>,

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
    fn reset(&self) {
        let mut action = self.action.write().unwrap();
        match self.reset_action.as_ref() {
            Some(a) => {
                *action = a.clone();
            }
            None => {
                *action = PinAction::highz();
            }
        }
    }

    pub fn add_metadata_id(&mut self, id_str: &str, id: usize) -> Result<()> {
        if self.metadata.contains_key(id_str) {
            Err(error!(
                "Pin {} already has metadata {}! Use set_metadata to override its current value!",
                self.name, id_str,
            ))
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

    pub fn new(model_id: usize, id: usize, name: String, reset_action: Option<PinAction>) -> Pin {
        let p = Pin {
            model_id: model_id,
            id: id,
            name: name,
            data: RwLock::new(0),
            action: RwLock::new(PinAction::highz()),
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
        Self::new(0, 0, String::from("default"), Option::None)
    }
}
