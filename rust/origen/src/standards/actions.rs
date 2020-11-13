use std::collections::HashMap;

pub static DRIVE_HIGH: &str = "1";
pub static DRIVE_LOW: &str = "0";
pub static VERIFY_HIGH: &str = "H";
pub static VERIFY_LOW: &str = "L";
pub static HIGHZ: &str = "Z";
pub static CAPTURE: &str = "C";

pub fn standard_actions() -> HashMap<String, String> {
    crate::hashmap!(
        "1".to_string() => "DRIVE_HIGH".to_string(),
        "0".to_string() => "DRIVE_LOW".to_string(),
        "H".to_string() => "VERIFY_HIGH".to_string(),
        "L".to_string() => "VERIFY_LOW".to_string(),
        "Z".to_string() => "HIGHZ".to_string(),
        "C".to_string() => "CAPTURE".to_string()
    )
}

lazy_static! {
    pub static ref STANDARD_ACTIONS: HashMap<String, String> = standard_actions();
}
