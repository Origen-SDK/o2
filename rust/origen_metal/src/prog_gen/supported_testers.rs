use crate::Result as OrigenResult;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Copy)]
pub enum SupportedTester {
    /// Generally, the absence of an optional SupportedTester value means all testers, but
    /// this can also be used to indicate that whenever a SupportedTester value is required
    ALL,
    /// Indicates support for all versions of SMT on the V93K
    V93K,
    /// Indicates support for V93K SMT7 only, i.e. not SMT8
    V93KSMT7,
    /// Indicates support for V93K SMT8 only, i.e. not SMT7
    V93KSMT8,
    /// Indicates support for all IGXL-based testers
    IGXL,
    J750,
    ULTRAFLEX,
    SIMULATOR,
}

impl SupportedTester {
    /// Returns the names of all available testers
    pub fn all_names() -> Vec<String> {
        let s = vec![
            "ALL",
            "V93K",
            "V93KSMT7",
            "V93KSMT8",
            "IGXL",
            "J750",
            "ULTRAFLEX",
            "SIMULATOR",
        ];
        let s: Vec<String> = s.iter().map(|n| n.to_string()).collect();
        s
    }

    pub fn new(name: &str) -> OrigenResult<Self> {
        match SupportedTester::from_str(name) {
            Ok(n) => Ok(n),
            Err(msg) => bail!("{}", msg),
        }
    }

    /// Returns true if the given tester is equal to self or if the given tester is a
    /// derivative of self (see is_a_derivative_of()).
    pub fn is_compatible_with(&self, tester: &SupportedTester) -> bool {
        self == tester || tester.is_a_derivative_of(self)
    }

    /// Returns true if self is a derivative of the given tester. For example if the given
    /// tester is IGXL, then both the J750 and the ULTRAFLEX would be considered derivatives
    /// of it.
    /// Note that true is only returned for derivatives, if the given tester is the same as
    /// self then false will be returned.
    /// Use is_compatible_with() if you also want to consider an exact match as true.
    pub fn is_a_derivative_of(&self, tester: &SupportedTester) -> bool {
        match self {
            SupportedTester::ALL => tester != &SupportedTester::ALL,
            SupportedTester::IGXL => {
                tester == &SupportedTester::J750 || tester == &SupportedTester::ULTRAFLEX
            }
            SupportedTester::V93K => {
                tester == &SupportedTester::V93KSMT7 || tester == &SupportedTester::V93KSMT8
            }
            _ => false,
        }
    }
}

impl FromStr for SupportedTester {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s.contains("::") {
            true => {
                let fields: Vec<&str> = s.split("::").collect();
                if fields.len() > 2 {
                    return Err(error_msg(&s));
                }
                fields[0]
            }
            false => s,
        };

        // Accept any case and with or without underscores
        let kind = kind.to_uppercase().replace("_", "");
        match kind.trim() {
            "ALL" | "ANY" => Ok(SupportedTester::ALL),
            "V93K" => Ok(SupportedTester::V93K),
            "V93KSMT7" => Ok(SupportedTester::V93KSMT7),
            "V93KSMT8" => Ok(SupportedTester::V93KSMT8),
            "IGXL" => Ok(SupportedTester::IGXL),
            "J750" => Ok(SupportedTester::J750),
            "ULTRAFLEX" | "UFLEX" => Ok(SupportedTester::ULTRAFLEX),
            "SIMULATOR" => Ok(SupportedTester::SIMULATOR),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn verify_custom_testers_work_as_hash_keys() {
        let mut h: HashMap<SupportedTester, usize> = HashMap::new();
        let t1 = SupportedTester::J750;

        h.insert(t1.clone(), 1);

        assert_eq!(h[&t1], 1);
    }
}
