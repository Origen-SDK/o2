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

    pub fn init_parsed(&mut self, allow_parsed: bool) -> Result<bool> {
        if self.parsed.is_some() {
            if allow_parsed {
                return Ok(false);
            } else {
                bail!("ParamStr has already been parsed. Please use 'param_str.clear()', before parsing new input");
            }
        }
        let mut parsed = IndexMap::new();
        if let Some(defs) = self.defaults.as_ref() {
            for (name, def_val) in &defs.1 {
                if let Some(v) = def_val {
                    parsed.insert(name.to_owned(), v.to_owned());
                }
            }
        }
        self.parsed = Some(parsed);
        Ok(true)
    }

    pub fn parse(&mut self, input: String) -> Result<&Self> {
        if self.parsed.is_some() {
            bail!("ParamStr has already been parsed. Please use 'param_str.clear()', before parsing new input");
        }
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

    fn param_set(&mut self, param: String, val: Option<Vec<String>>) -> Result<bool> {
        match self.parsed.as_mut() {
            Some(params) => {
                let retn = params.contains_key(&param);
                if let Some(defs) = self.defaults.as_ref() {
                    if (!defs.0) && (!defs.1.contains_key(&param)) {
                        bail!("ParamStr encountered parameter '{}', which is not an expected parameter", &param);
                    }
                }
                if let Some(v) = val {
                    params.insert(param, v);
                } else {
                    params.remove(&param);
                }
                Ok(retn)
            },
            None => bail!(*NOT_PARSED_MSG)
        }
    }

    pub fn set_param(&mut self, param: String, val: Option<Vec<String>>) -> Result<bool> {
        if self.parsed.is_some() {
            self.param_set(param, val)
        } else {
            self.init_parsed(true)?;
            self.param_set(param, val)
        }
    }

    pub fn defaults(&self) -> Option<&IndexMap<String, Option<Vec<String>>>> {
        if let Some(defs) = self.defaults.as_ref() {
            Some(&defs.1)
        } else {
            None
        }
    }

    pub fn with_mut_defs<T, F>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut (bool, IndexMap<String, Option<Vec<String>>>)) -> Result<T>,
    {
        if self.parsed.is_some() {
            bail!("Attempted to update ParamStr's default values after parsing, which is not allowed")
        }
        if self.defaults.is_none() {
            self.defaults = Some((false, IndexMap::new()));
        }
        f(self.defaults.as_mut().unwrap())
    }

    pub fn add_default(&mut self, def: String, value: Option<Vec<String>>) -> Result<bool> {
        self.with_mut_defs( |defs| {
            let retn = defs.1.contains_key(&def);
            defs.1.insert(def, value);
            Ok(retn)
        })
    }

    pub fn add_defaults(&mut self, to_add: IndexMap<String, Option<Vec<String>>>) -> Result<Vec<bool>> {
        self.with_mut_defs( |defs| {
            let mut retn = vec!();
            for (name, val) in to_add {
                retn.push(defs.1.contains_key(&name));
                defs.1.insert(name, val);
            }
            Ok(retn)
        })
    }

    pub fn remove_default(&mut self, to_remove: &str) -> Result<Option<Vec<String>>> {
        self.with_mut_defs( |defs| {
            match defs.1.shift_remove(to_remove) {
                Some(value) => Ok(value),
                None => bail!("No parameter '{}' to remove from ParamStr's defaults", to_remove)
            }
        })
    }

    pub fn remove_defaults(&mut self, to_remove: &Vec<String>) -> Result<Vec<Option<Vec<String>>>> {
        self.with_mut_defs( |defs| {
            // Check that all keys are valid first
            for name in to_remove {
                if !defs.1.contains_key(name) {
                    bail!("No parameter '{}' to remove from ParamStr's defaults", name)
                }
            }

            let mut retn = vec!();
            for name in to_remove {
                retn.push(defs.1.shift_remove(name).unwrap());
            }
            Ok(retn)
        })
    }

    pub fn allows_non_defaults(&self) -> Option<bool> {
        if let Some(setup) = self.defaults.as_ref() {
            Some(setup.0)
        } else {
            None
        }
    }

    pub fn set_allows_non_defaults(&mut self, new_val: bool) -> Result<()> {
        if self.parsed.is_some() {
            bail!("Cannot set ParamStr's allows_non_defaults with no default parameters");
        }
        self.with_mut_defs( |defs| {
            defs.0 = new_val;
            Ok(())
        })
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

    pub fn clear(&mut self) -> Result<&mut Self> {
        self.parsed = None;
        self.leading = None;
        self.raw = None;
        Ok(self)
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

    pub fn set_leading(&mut self, new_leading: Option<String>) -> Result<bool> {
        if !self.allows_leading_str {
            bail!("Attempted to set leading value but 'allows_leading_str' is not allowed")
        }
        self.init_parsed(true)?;
        let retn = self.leading.is_some();
        self.leading = new_leading;
        Ok(retn)
    }

    pub fn allows_leading_str(&self) -> bool {
        self.allows_leading_str
    }

    pub fn set_allows_leading_str(&mut self, new_val: bool) -> Result<()> {
        if self.parsed.is_some() {
            bail!("Attempted to change ParamStr's 'allows_leading_str' setting after parsing, which is not allowed");
        }
        self.allows_leading_str = new_val;
        Ok(())
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
            let params = args.iter().map(|(k, v)| {
                if v.is_empty() {
                    k.clone()
                } else {
                    format!("{}:{}", k, v.join(":"))
                }
            }).collect::<Vec<String>>().join("~");
            if let Some(leading) = self.leading.as_ref() {
                if params.is_empty() {
                    Ok(leading.clone())
                } else {
                    Ok(format!("{}~{}", leading, params))
                }
            } else {
                Ok(params)
            }
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