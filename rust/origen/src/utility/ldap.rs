use ldap3::{LdapConn, Scope, SearchEntry};
use crate::Result;
use std::collections::HashMap;

pub struct LDAPs {
    ldaps: HashMap<String, LDAP>
}

impl LDAPs {
    pub fn get(&self, ldap: &str) -> Option<&LDAP> {
        self.ldaps.get(ldap)
    }

    pub fn _get(&self, ldap: &str) -> Result<&LDAP> {
        if let Some(l) = self.ldaps.get(ldap) {
            Ok(l)
        } else {
            error!(
                "No LDAP named {} available",
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

    pub fn add(&mut self, name: &str, server: &str, base: &str, auth: SupportedAuths) -> Result<()> {
        self.ldaps.insert(name.to_string(), LDAP::new(
            name,
            server,
            base,
            auth
        )?);
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut ldaps = Self::default();
        for (name, config) in &crate::ORIGEN_CONFIG.ldaps {
            ldaps.add(
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
                SupportedAuths::from_str(
                    config.get("auth").unwrap_or(&"sipmle_bind".to_string()),
                    config.get("username"),
                    config.get("password")
                )?
            )?;
        }
        Ok(ldaps)
    }
}

impl std::default::Default for LDAPs {
    fn default() -> Self {
        Self {
            ldaps: HashMap::new()
        }
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
                        return error!("LDAP's 'sipmle bind' requires a username but none was provided");
                    },
                    if let Some(p) = password {
                        p.to_string()
                    } else {
                        return error!("LDAP's 'sipmle_bind' requires a password but none was provided");
                    }
                ))
            },
            _ => error!("Unrecognized LDAP authentication {}", auth)
        }
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
    auth: SupportedAuths
}

impl LDAP {
    pub fn new(name: &str, server: &str, base: &str, auth: SupportedAuths) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            server: server.to_string(),
            base: base.to_string(),
            ldap: LdapConn::new(server)?,
            auth: auth
        })
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

    pub fn bind(&mut self) -> Result<()> {
        self.auth.bind(&mut self.ldap)
    }

    pub fn unbind(&mut self) -> Result<()> {
        self.ldap.unbind()?;
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
