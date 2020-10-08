use crate::Result as OrigenResult;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum SupportedTester {
    V93KSMT7,
    V93KSMT8,
    J750,
    ULTRAFLEX,
    SIMULATOR,
    // These two are only available in an Origen workspace
    DUMMYRENDERER,
    DUMMYRENDERERWITHINTERCEPTORS,
    // Used to identify an app-defined tester (in Python)
    CUSTOM(String),
}

impl SupportedTester {
    /// Returns the names of all available testers
    pub fn all_names() -> Vec<String> {
        let mut s = vec!["V93KSMT7", "V93KSMT8", "J750", "ULTRAFLEX", "SIMULATOR"];
        if crate::STATUS.is_origen_present {
            s.push("DUMMYRENDERER");
            s.push("DUMMYRENDERERWITHINTERCEPTORS");
        }
        let mut s: Vec<String> = s.iter().map(|n| n.to_string()).collect();
        for id in crate::STATUS.custom_tester_ids() {
            s.push(format!(", CUSTOM::{}", id));
        }
        s
    }

    pub fn new(name: &str) -> OrigenResult<Self> {
        match SupportedTester::from_str(name) {
            Ok(n) => Ok(n),
            Err(msg) => error!("{}", msg),
        }
    }
}

impl FromStr for SupportedTester {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut second: Option<String> = None;
        let kind = match s.contains("::") {
            true => {
                let fields: Vec<&str> = s.split(s).collect();
                if fields.len() > 2 {
                    return Err(error_msg(&s));
                }
                second = Some(fields[1].to_string());
                fields[0]
            }
            false => s,
        };

        // Accept any case and with or without underscores
        let kind = kind.to_uppercase().replace("_", "");
        match kind.as_str() {
            "V93KSMT7" => Ok(SupportedTester::V93KSMT7),
            "V93KSMT8" => Ok(SupportedTester::V93KSMT8),
            "J750" => Ok(SupportedTester::J750),
            "ULTRAFLEX" | "UFLEX" => Ok(SupportedTester::ULTRAFLEX),
            "SIMULATOR" => Ok(SupportedTester::SIMULATOR),
            "DUMMYRENDERER" => Ok(SupportedTester::DUMMYRENDERER),
            "DUMMYRENDERERWITHINTERCEPTORS" => Ok(SupportedTester::DUMMYRENDERERWITHINTERCEPTORS),
            "CUSTOM" => {
                if let Some(n) = second {
                    Ok(SupportedTester::CUSTOM(n))
                } else {
                    Err(error_msg(&s))
                }
            }
            _ => Err(error_msg(&s)),
        }
    }
}

impl fmt::Display for SupportedTester {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn error_msg(val: &str) -> String {
    format!(
        "'{}' is not a valid tester type, the available testers are: {}",
        val,
        SupportedTester::all_names().join(", ")
    )
}
