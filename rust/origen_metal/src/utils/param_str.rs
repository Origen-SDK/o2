use indexmap::IndexMap;
use crate::Result;
use lazy_static::lazy_static;

lazy_static! {
    static ref MULTI_PARAM_SEP: &'static str = "~:~";
    static ref NOT_PARSED_MSG: &'static str = "ParamStr has not yet been parsed!";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamStr {
    raw: Option<String>,
    parsed: Option<IndexMap<String, Vec<String>>>,
    leading: Option<String>,

    // Setup pieces
    allows_leading_str: bool,
    defaults: Option<(bool, IndexMap<String, Option<Vec<String>>>)>,
}

impl ParamStr {
    pub fn new(allows_leading_str: bool, defaults: Option<(bool, IndexMap<String, Option<Vec<String>>>)>) -> Self {
        Self {
            raw: None,
            parsed: None,
            leading: None,
            allows_leading_str,
            defaults,
        }
    }

    pub fn parse(&mut self, input: String) -> Result<&Self> {
        let input_inter;
        let leading;
        let mut parsed = IndexMap::new();

        if input.contains(*MULTI_PARAM_SEP) {
            bail!("ParamStr input contains the multi-param-separate '{}', which is not allowed. Please use a MultiParamStr instead", *MULTI_PARAM_SEP);
        }

        // Handle the leading string
        if self.allows_leading_str {
            if let Some(split) = input.split_once("~") {
                leading = Some(split.0.to_string());
                input_inter = split.1.to_string();
            } else {
                leading = Some(input.clone());
                input_inter = "".to_string();
            }
        } else {
            leading = None;
            input_inter = input.clone();
        }

        if !input_inter.is_empty() {
            let allows_non_defaults;
            if let Some(defs) = self.defaults.as_ref() {
                allows_non_defaults = defs.0;
            } else {
                allows_non_defaults = true;
            }

            if input_inter.trim().starts_with(":") {
                if leading.is_some() {
                    // Something like blah~: was given. Use empty key message
                    bail!("ParamStr encountered a parameter with an empty key, which is not allowed");
                } else {
                    bail!("ParamStr found value separator as first character, which is not allowed");
                }
            }

            let mut found_leading_colon = false;
            for param_str in input_inter.split('~') {
                if found_leading_colon {
                    if param_str == "~" {
                        // encountered ~:~, which not allowed in a single param string
                        bail!("Encountered ParamStr split sequence ~:~ in a single ParamStr, which is not allowed. Please use a MultiParamStr or update the input for a single ParamStr");
                    } else {
                        // encountered ~:, which does not have a param key and not allowed
                        bail!("ParamStr encountered a parameter with a value separator but no key, which is not allowed");
                    }
                }
                if param_str == ":" {
                    found_leading_colon = true;
                    continue;
                } else if param_str.is_empty() {
                    bail!("ParamStr encountered a parameter with an empty key, which is not allowed")
                } else if param_str.trim().is_empty() {
                    bail!("ParamStr encountered a parameter of only whitespace, which is not allowed")
                }

                let k;
                macro_rules! check_key {
                    () => {
                        if !allows_non_defaults {
                            if !self.defaults.as_ref().unwrap().1.contains_key(&k) {
                                bail!("ParamStr encountered parameter '{}', which is not an expected parameter", k);
                            }
                        }
                        if parsed.contains_key(&k) {
                            bail!("ParamStr encountered a duplicate parameter '{}', which is not allowed", k);
                        }
                    }
                }

                let key_split = param_str.split_once(":");
                if let Some(split) = key_split {
                    k = split.0.to_string();
                    check_key!();
                    parsed.insert(
                        k,
                        split.1.split(":").map(|s| s.to_string()).collect::<Vec<String>>()
                    );
                } else {
                    k = param_str.to_owned();
                    check_key!();
                    parsed.insert(k, vec!());
                }
            }
            if found_leading_colon {
                bail!("ParamStr encountered a parameter with a value separator but no key, which is not allowed");
            }
        }

        // Fill in any missing defaults
        if let Some(defs) = self.defaults.as_ref() {
            for (n, def) in &defs.1 {
                if !parsed.contains_key(n) {
                    if let Some(d) = def {
                        parsed.insert(n.to_owned(), d.to_owned());
                    }
                }
            }
        }

        self.parsed = Some(parsed);
        self.leading = leading;
        self.raw = Some(input);
        Ok(self)
    }

    pub fn defaults(&self) -> Option<&IndexMap<String, Option<Vec<String>>>> {
        if let Some(defs) = self.defaults.as_ref() {
            Some(&defs.1)
        } else {
            None
        }
    }

    pub fn allows_non_defaults(&self) -> Option<bool> {
        if let Some(setup) = self.defaults.as_ref() {
            Some(setup.0)
        } else {
            None
        }
    }

    pub fn parsed(&self) -> &Option<IndexMap<String, Vec<String>>> {
        &self.parsed
    }

    pub fn get_parsed(&self) -> Result<&IndexMap<String, Vec<String>>> {
        if let Some(parsed) = self.parsed.as_ref() {
            Ok(parsed)
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn raw(&self) -> Result<&Option<String>> {
        if self.parsed.is_some() {
            Ok(&self.raw)
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn leading(&self) -> Result<&Option<String>> {
        if self.parsed.is_some() {
            Ok(&self.leading)
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn allows_leading_str(&self) -> bool {
        self.allows_leading_str
    }
    
    pub fn get(&self, key: &str) -> Result<Option<&Vec<String>>> {
        if let Some(args) = self.parsed.as_ref() {
            Ok(args.get(key))
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn to_string(&self) -> Result<String> {
        if let Some(args) = self.parsed.as_ref() {
            Ok(args.iter().map(|(k, v)| {
                if v.is_empty() {
                    k.clone()
                } else {
                    format!("{}:{}", k, v.join(":"))
                }
            }).collect::<Vec<String>>().join("~"))
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn keys(&self) -> Result<Vec<&String>> {
        if let Some(args) = self.parsed.as_ref() {
            Ok(args.keys().collect())
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }

    pub fn values(&self) -> Result<Vec<&Vec<String>>> {
        if let Some(args) = self.parsed.as_ref() {
            Ok(args.values().collect())
        } else {
            bail!(*NOT_PARSED_MSG);
        }
    }
}

pub struct MultiParamStr {
    param_strs: Vec<ParamStr>,
    allows_leading_str: bool,

    raw: Option<String>,
    parsed: Option<Vec<ParamStr>>,
    leading: Option<String>,
}

impl MultiParamStr {
    pub fn new(allows_leading_str: bool) -> Self {
        Self {
            param_strs: vec!(),
            allows_leading_str: allows_leading_str,

            raw: None,
            parsed: None,
            leading: None,
        }
    }

    pub fn new_with_leading() -> Self {
        Self {
            param_strs: vec!(),
            allows_leading_str: true,

            raw: None,
            parsed: None,
            leading: None,
        }
    }

    pub fn new_without_leading() -> Self {
        Self {
            param_strs: vec!(),
            allows_leading_str: false,

            raw: None,
            parsed: None,
            leading: None,
        }
    }

    pub fn parse(&mut self, input_str: String) -> Result<()> {
        let params: &str;
        let leading: Option<String>;
        let mut parsed: Vec<ParamStr> = vec!();

        if self.allows_leading_str {
            if let Some(split) = input_str.split_once(*MULTI_PARAM_SEP) {
                leading = Some(split.0.to_string());
                params = split.1;
            } else {
                leading = Some(input_str.to_string());
                params = "";
            }
        } else {
            leading = None;
            params = input_str.as_str();
        }

        if !params.is_empty() {
            // TODO support parsing into given ParamStrs
            for (_i, param_str) in params.split(*MULTI_PARAM_SEP).enumerate() {
                if self.param_strs.is_empty() {
                    // Individual ParamStrs from a multi-param string cannot have leading strings
                    let mut p = ParamStr::new(false, None);
                    p.parse(param_str.to_string())?;
                    parsed.push(p);
                } else {
                    todo!("Named ParamStrs not ready yet!");
                }
            }
        }

        self.leading = leading;
        self.parsed = Some(parsed);
        self.raw = Some(input_str);
        Ok(())
    }

    pub fn allows_leading_str(&self) -> bool {
        self.allows_leading_str
    }

    pub fn param_strs(&self) -> &Vec<ParamStr> {
        &self.param_strs
    }

    pub fn parsed(&self) -> &Option<Vec<ParamStr>> {
        &self.parsed
    }

    pub fn leading(&self) -> &Option<String> {
        &self.leading
    }

    pub fn raw(&self) -> &Option<String> {
        &self.raw
    }
}