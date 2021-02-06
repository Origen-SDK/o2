use ldap3::{LdapConn, Scope, SearchEntry};
use crate::Result;
use std::collections::HashMap;

pub struct LDAPs {
    ldaps: HashMap<String, LDAP>
}

impl LDAPs {
    pub fn get_standalone(ldap_name: &str) -> Result<LDAP> {
        LDAP::from_config(ldap_name)
    }

    pub fn get(&self, ldap: &str) -> Option<&LDAP> {
        self.ldaps.get(ldap)
    }

    pub fn _get(&self, ldap: &str) -> Result<&LDAP> {
        if let Some(l) = self.ldaps.get(ldap) {
            Ok(l)
        } else {
            error!(
                "No LDAP named '{}' available",
                ldap
            )
        }
    }

    pub fn get_mut(&mut self, ldap: &str) -> Option<&mut LDAP> {
        self.ldaps.get_mut(ldap)
    }

    pub fn _get_mut(&mut self, ldap: &str) -> Result<&mut LDAP> {
        if let Some(l) = self.ldaps.get_mut(ldap) {
            Ok(l)
        } else {
            error!(
                "No LDAP named {} available",
                ldap
            )
        }
    }

    pub fn ldaps(&self) -> &HashMap<String, LDAP> {
        &self.ldaps
    }

    pub fn new() -> Self {
        let mut ldaps: HashMap<String, LDAP> = HashMap::new();
        for (name, _) in &crate::ORIGEN_CONFIG.ldaps {
            match LDAP::from_config(name) {
                Ok(l) => {
                    ldaps.insert(name.to_string(), l);
                },
                Err(e) => {
                    display_redln!("Unable to add LDAP {}. Reason: {}", name, e.msg);
                }
            }
        }
        Self {
            ldaps: ldaps
        }
    }

    pub fn try_password(ldap_name: &str, username: &str, password: &str) -> Result<bool> {
        let mut ldap = Self::get_standalone(ldap_name)?;
        match ldap.bind_as(username, password) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.msg.contains("rc=49") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn with_standalone<T, F>(name: &str, mut func: F) -> Result<T>
    where F: FnMut(LDAP) -> Result<T> {
        let ldap = Self::get_standalone(&name)?;
        func(ldap)
    }
}

// Can be extended as other LDAP auth support is needed
#[derive(Debug, Clone)]
pub enum SupportedAuths {
    SimpleBind(String, String) // Username/Password
}

impl SupportedAuths {
    pub fn to_str(&self) -> &str {
        match self {
            Self::SimpleBind(_, _) => "simple_bind"
        }
    }

    pub fn from_str(auth: &str, username: Option<&String>, password: Option<&String>) -> Result<Self> {
        match auth {
            "simple" | "Simple" | "simple_bind" | "SimpleBind" => {
                Ok(Self::SimpleBind(
                    if let Some(u) = username {
                        u.to_string()
                    } else {
                        return error!("LDAP's 'simple bind' requires a username but none was provided");
                    },
                    if let Some(p) = password {
                        p.to_string()
                    } else {
                        return error!("LDAP's 'simple_bind' requires a password but none was provided");
                    }
                ))
            },
            _ => error!("Unrecognized LDAP authentication {}", auth)
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut retn = HashMap::new();
        retn.insert("type".to_string(), self.to_str().to_string());
        match self {
            Self::SimpleBind(username, password) => {
                retn.insert("username".to_string(), username.to_string());
                retn.insert("password".to_string(), password.to_string());
            }
        }
        retn
    }

    pub fn bind(&self, ldap: &mut LdapConn) -> Result<()> {
        match self {
            Self::SimpleBind(username, password) => {
                ldap.simple_bind(
                    username,
                    password
                )?.success()?;
                Ok(())
            }
        }
    }
}

pub struct LDAP {
    name: String,
    server: String,
    base: String,
    ldap: LdapConn,
    auth: SupportedAuths,
    bound: bool
}

impl LDAP {
    pub fn new(name: &str, server: &str, base: &str, auth: SupportedAuths) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            server: server.to_string(),
            base: base.to_string(),
            ldap: LdapConn::new(server)?,
            auth: auth,
            bound: false,
        })
    }

    pub fn from_config(name: &str) -> Result<Self> {
        if let Some(config) = crate::ORIGEN_CONFIG.ldaps.get(name) {
            Self::new(
                name,
                if let Some(c) = config.get("server") {
                    c
                } else {
                    return error!("LDAP config {} must have a 'server' field", name);
                },
                if let Some(b) = config.get("base") {
                    b
                } else {
                    return error!("LDAP config {} must have a 'base' field", name);
                },
                {
                    let (username, password);
                    if let Some(u) = config.get("service_user") {
                        let su = crate::ORIGEN_CONFIG.get_service_user(u)?;
                        if let Some(_su) = su {
                            username = _su.get("username");
                            password = _su.get("password");
                        } else {
                            return error!("Could not find service user {}", u);
                        }
                    } else {
                        username = config.get("username");
                        password = config.get("password");
                    }
                    SupportedAuths::from_str(
                        config.get("auth").unwrap_or(&"simple_bind".to_string()),
                        username,
                        password
                    )?
                }
            )
        } else {
            error!("No ldap {} found in the configuration", name)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn auth(&self) -> &SupportedAuths {
        &self.auth
    }

    pub fn bound(&self) -> bool {
        self.bound
    }

    pub fn bind(&mut self) -> Result<()> {
        // Doesn't seem to be any harm in binding again. Probably remove this
        // to speed up accesses later
        self.auth.bind(&mut self.ldap)?;
        self.bound = true;
        Ok(())
    }

    pub fn bind_as(&mut self, username: &str, password: &str) -> Result<()> {
        self.unbind()?;
        self.auth = SupportedAuths::SimpleBind(username.to_string(), password.to_string());
        self.auth.bind(&mut self.ldap)?;
        self.bound = true;
        Ok(())
    }

    pub fn bind_with(&mut self, auth: SupportedAuths) -> Result<()> {
        self.unbind()?;
        self.auth = auth;
        self.auth.bind(&mut self.ldap)?;
        self.bound = true;
        Ok(())
    }

    pub fn unbind(&mut self) -> Result<()> {
        self.ldap.unbind()?;
        self.bound = false;
        self.ldap = LdapConn::new(&self.server)?;
        Ok(())
    }

    pub fn search(
        &mut self,
        filter: &str,
        attrs: Vec<&str>
    ) -> Result<HashMap<String, (
        HashMap<String, Vec<String>>,
        HashMap<String, Vec<Vec<u8>>>
    )>> {
        self.bind()?;
        let (rs, _result) = self.ldap.search(
            &self.base,
            Scope::Subtree,
            filter,
            attrs
        )?.success()?;
        let mut retn = HashMap::new();
        for entry in rs {
            let construct = SearchEntry::construct(entry);
            retn.insert(
                construct.dn,
                (construct.attrs, construct.bin_attrs)
            );
        }
        Ok(retn)
    }

    pub fn single_filter_search(
        &mut self,
        filter: &str,
        attrs: Vec<&str>
    ) -> Result<(
        HashMap<String, Vec<String>>,
        HashMap<String, Vec<Vec<u8>>>
    )> {
        self.bind()?;
        let (mut rs, _result) = self.ldap.search(
            &self.base,
            Scope::Subtree,
            filter,
            attrs
        )?.success()?;
        if rs.len() > 1 {
            return error!(
                "LDAP: expected a single DN result from filter {} for 'single_filter_search'. \
                Use 'search' if multiple DN entries were expected.",
                filter
            );
        } else if rs.len() == 0 {
            Ok((HashMap::new(), HashMap::new()))
        } else {
            let construct = SearchEntry::construct(rs.pop().unwrap());
            Ok((construct.attrs, construct.bin_attrs))
        }
    }
}
