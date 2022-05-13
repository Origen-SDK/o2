use crate::{Result, Outcome, TypedValue, TypedValueMap};
use crate::framework::users::{User, Data};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use std::collections::HashMap;
use std::sync::{RwLock, RwLockWriteGuard};
use core::time::Duration;

#[derive(Debug, Clone)]
pub struct SimpleBind {
    pub username: Option<String>,
    pub password: Option<String>,
    pub priority_motives: Vec<String>,
    pub backup_motives: Vec<String>,
    pub allow_default_password: bool,
    pub use_default_motives: bool
}

impl SimpleBind {
    pub fn username(&self) -> Result<String> {
        if let Some(u) = &self.username {
            Ok(u.to_owned())
        } else {
            crate::require_current_user_id()
        }
    }

    pub fn motives<'a>(&'a self, ldap_name: &'a str) -> Vec<&'a str> {
        let mut retn = self.priority_motives.iter().map( |s| s.as_str()).collect::<Vec<&str>>();
        if self.use_default_motives {
            retn.append(&mut vec![ldap_name, "ldap"]);
        }
        retn.extend(self.backup_motives.iter().map( |s| s.as_str()).collect::<Vec<&str>>());
        retn
    }

    pub fn password(&self, ldap_name: &str) -> Result<String> {
        if let Some(p) = &self.password {
            Ok(p.to_owned())
        } else {
            crate::with_user(&self.username()?, |u| {
                for m in &self.motives(ldap_name) {
                    if let Some(d) = u.dataset_for(m)? {
                        return u.password(Some(d), false, None)
                    }
                }
                if self.allow_default_password {
                    u.password(None, false, None)
                } else {
                    bail!("No password found for user '{}' matching motives {}", u.id(), self.motives(ldap_name).iter().map( |m| format!("'{}'", m)).collect::<Vec<String>>().join(", "))
                }
            })
        }
    }

    // TODO
//     pub fn update(
//         &mut self,
//         username: Option<Option<String>>,
//         password: Option<Option<String>>,
//         priority_motives: Option<Vec<String>>,
//         backup_motives: Option<Vec<String>>,
//         allow_default_password: Option<bool>,
//         use_default_motives: Option<bool>,
//     ) -> Result<()> {
//         if (password.is_none() && self.password.is_none()) || (password.unwrap().is_none())
//             && (priority_motives.is_empty() && priority_motives.is_empty())

//         if password.is_none() && priority_motives.is_empty() && backup_motives.is_empty() && !allow_default_password && !use_default_motives {
//             bail!("Password is unresolvable! Current config does not provide a password, any motives, nor are any default motives or passwords allowed!");
//         }

//         if let Some(u) = username {
//             self.username = u;
//         }
//         if let Some(p) = password {
//             self.password = p;
//         }
//         if let Some(pm) = priority_motives {
//             self.priority_motives = u;
//         }
//         if let Some(bm) = backup_motives {
//             self.backup_motives = u;
//         }
//         if let Some(a) = allow_default_password {
//             self.allow_default_password = a;
//         }
//         if let Some(u) = use_default_motives {
//             self.use_default_motives = u;
//         }
//         Ok(())
//     }
}

impl std::default::Default for SimpleBind {
    fn default() -> Self {
        Self {
            username: None,
            password: None,
            priority_motives: vec![],
            backup_motives: vec![],
            allow_default_password: true,
            use_default_motives: true
        }
    }
}

// Can be extended as other LDAP auth support is needed
#[derive(Debug, Clone)]
pub enum SupportedAuths {
    SimpleBind(SimpleBind),
}

impl SupportedAuths {
    pub fn to_str(&self) -> &str {
        match self {
            Self::SimpleBind(_) => "simple_bind",
        }
    }

    pub fn from_str(auth: &str) -> Result<Self> {
        match auth.to_lowercase().as_str() {
            "simple" | "simple_bind" | "simplebind" => Ok(Self::SimpleBind(SimpleBind::default())),
            _ => bail!("Unrecognized LDAP authentication {}", auth),
        }
    }

    pub fn resolve_and_into_map(&self, l: &LDAP) -> Result<TypedValueMap> {
        let mut tvm = TypedValueMap::new();
        match self {
            Self::SimpleBind(sb) => {
                tvm.insert("scheme", self.to_str());
                tvm.insert("username", sb.username()?);
                tvm.insert("password", sb.password(&l.name)?);
                tvm.insert("motives", sb.motives(&l.name));
            }
        }
        Ok(tvm)
    }

    pub fn config_into_map(&self) -> TypedValueMap {
        let mut tvm = TypedValueMap::new();
        tvm.insert("scheme", self.to_str());
        match self {
            Self::SimpleBind(sb) => {
                tvm.insert("username", sb.username.as_ref());
                tvm.insert("password", sb.password.as_ref());
                tvm.insert("priority_motives", sb.priority_motives.iter());
                tvm.insert("backup_motives", sb.backup_motives.iter());
                tvm.insert("allow_default_password", sb.allow_default_password);
                tvm.insert("use_default_motives", sb.use_default_motives);
            }
        }
        tvm
    }

    pub fn bind(&self, l: &LDAP, ldap: &mut LdapConn) -> Result<()> {
        // TODO customize this
        ldap.with_timeout(core::time::Duration::new(5, 0));
        match self {
            Self::SimpleBind(sb) => {
                ldap.simple_bind(&sb.username()?, &sb.password(l.name.as_str())?)?.success()?;
                Ok(())
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LdapPopUserConfig {
    pub data_id: String,
    pub mapping: HashMap<String, String>,
    // TODO support some of these options. Currently unsupported
    pub required: Vec<String>,
    pub include_all: bool,
    pub attributes: Option<Vec<String>>,
}

impl LdapPopUserConfig {
    pub fn get_attributes(&self) -> Vec<&str> {
        match &self.attributes {
            Some(attrs) => attrs.iter().map( |a| a.as_str()).collect(),
            None => vec!["*"]
        }
    }

    // TODO
    pub fn config_into_map(&self) -> TypedValueMap {
        todo!();
        // let mut tvm = TypedValueMap::new();
        // tvm.insert("data_id", &self.data_id);
        // tvm.insert("mapping", self.mapping.iter());
        // // tvm.insert("required", self.required.iter());
        // // tvm.insert("include_all", self.include_all);
        // // tvm.insert("attributes", self.attributes.iter());
        // tvm
    }
}

impl Default for LdapPopUserConfig {
    fn default() -> Self {
        Self {
            data_id: "uid".to_string(),
            mapping: HashMap::new(),
            required: vec![],
            include_all: false,
            attributes: None
        }
    }
}

pub struct LDAP {
    name: String,
    server: String,
    base: String,
    ldap: RwLock<LdapConn>,
    auth: SupportedAuths,
    bound: RwLock<bool>,
    continuous_bind: bool,
    populate_user_config: Option<LdapPopUserConfig>,
    timeout: Option<Duration>,
}

impl LDAP {
    pub fn new<S: Into<u64>>(name: &str, server: &str, base: &str, continuous_bind: bool, auth: SupportedAuths, timeout: Option<Option<S>>, populate_user_config: Option<LdapPopUserConfig>) -> Result<Self> {
        let t = match timeout {
            Some(t) => match t {
                Some(t2) => Some(Duration::new(t2.into(), 0)),
                None => None
            },
            None => Some(Duration::new(60 as u64, 0))
        };

        let s = Self {
            name: name.to_string(),
            server: server.to_string(),
            base: base.to_string(),
            auth: auth,

            // Many of the operations change the state of the LdapConn, requiring a mutable reference to it.
            // Wrap this in a RwLock to provide thread-safe interior mutability to hide this detail.
            // Otherwise, most operations would require mutable references, but don't want to force this.
            ldap: RwLock::new({
                let mut settings = LdapConnSettings::new();
                if let Some(t_out) = t {
                    settings = settings.set_conn_timeout(t_out);
                }
                LdapConn::with_settings(settings, server)?
            }),
            bound: RwLock::new(false),
            continuous_bind: continuous_bind,
            timeout: t,
            populate_user_config: populate_user_config,
        };
        Ok(s)
    }

    pub fn standalone(&self) -> Result<Self> {
        // TODO timeout
        Self::new(&self.name, &self.server, &self.base, self.continuous_bind, self.auth.clone(), Some(self.timeout()), self.populate_user_config.clone())
    }

    // pub fn timeout(&self) -> Option<u64> {
    //     match self.timeout {
    //         Some(t) => Some(t.as_secs()),
    //         None => None
    //     }
    // }

    // pub fn set_timeout<S: Into<u64>>(&self, secs: S) -> Result<()> {
    //     let mut ldap = self.ldap.write()?;
    //     // TODO
    //     // ldap.timeout = Duration::new(secs.into(), 0);
    //     Ok(())
    // }

    // // TODO
    // pub fn from_config(name: &str, config: LdapConfigType) -> Result<Self> {
    //     Self::new::<u64>(
    //         name,
    //         if let Some(c) = config.get("server") {
    //             c
    //         } else {
    //             bail!("LDAP config {} must have a 'server' field", name);
    //         },
    //         if let Some(b) = config.get("base") {
    //             b
    //         } else {
    //             bail!("LDAP config {} must have a 'base' field", name);
    //         },
    //         // TODO
    //         false,
    //         // if let Some(b) = config.get("continuous_bind") {
    //         //     b
    //         // } else {
    //         //     false
    //         // },
    //         {
    //             let (username, password);
    //             if let Some(_u) = config.get("service_user") {
    //                 // TODO add support for this again
    //                 todo!("LDAP service user not supported in metal yet!")
    //                 // let su = crate::ORIGEN_CONFIG.get_service_user(u)?;
    //                 // if let Some(_su) = su {
    //                 //     username = _su.get("username");
    //                 //     password = _su.get("password");
    //                 // } else {
    //                 //     bail!("Could not find service user {}", u);
    //                 // }
    //             } else {
    //                 username = config.get("username");
    //                 password = config.get("password");
    //             }
    //             SupportedAuths::from_str(
    //                 config.get("auth").unwrap_or(&"simple_bind".to_string()),
    //                 username,
    //                 password,
    //             )?
    //         },
    //         // TODO
    //         // None::<u64>,
    //         None,
    //         None,
    //     )
    // }

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
        *self.bound.read().unwrap()
    }

    pub fn timeout(&self) -> Option<u64> {
        self.timeout.map_or(None, |t| Some(t.as_secs()))
    }

    pub fn continuous_bind(&self) -> bool {
        self.continuous_bind
    }

    pub fn bind(&self) -> Result<()> {
        self.execute( |_| { Ok(()) })
    }

    fn bind_comm(&self, comm: &mut LdapConn) -> Result<()> {
        let bound = *self.bound.write()?;
        if !bound {
            self.auth.bind(&self, comm)?;
            *self.bound.write()? = true;
        }
        Ok(())
    }

    pub fn bind_as(&mut self, username: &str, password: &str) -> Result<()> {
        self.unbind()?;
        self.auth = SupportedAuths::SimpleBind({
            let mut sb = SimpleBind::default();
            sb.username = Some(username.to_string());
            sb.password = Some(password.to_string());
            sb
        });

        let mut comm = self.ldap.write()?;
        self.auth.bind(&self, &mut comm)?;
        *self.bound.write()? = true;
        Ok(())
    }

    pub fn bind_with(&mut self, auth: SupportedAuths) -> Result<()> {
        self.unbind()?;
        self.auth = auth;

        let mut comm = self.ldap.write()?;
        self.auth.bind(&self, &mut comm)?;
        *self.bound.write()? = true;
        Ok(())
    }

    pub fn unbind(&self) -> Result<bool> {
        let comm = self.ldap.write()?;
        self.unbind_comm(comm)
    }

    fn unbind_comm(&self, mut comm: RwLockWriteGuard<LdapConn>) -> Result<bool> {
        comm.unbind()?;

        *comm = LdapConn::new(&self.server)?;
        let mut bound = self.bound.write()?;
        let was_bound = *bound;
        *bound = false;
        Ok(was_bound)
    }

    pub fn search(
        &self,
        filter: &str,
        attrs: Vec<&str>,
    ) -> Result<HashMap<String, (HashMap<String, Vec<String>>, HashMap<String, Vec<Vec<u8>>>)>>
    {
        self.execute(|comm| {
            let (rs, _result) = comm.search(&self.base, Scope::Subtree, filter, attrs)?.success()?;
            let mut retn = HashMap::new();
            for entry in rs {
                let construct = SearchEntry::construct(entry);
                retn.insert(construct.dn, (construct.attrs, construct.bin_attrs));
            }
            Ok(retn)
        })
    }

    pub fn single_filter_search<S: AsRef<str> + Send + Sync>(
        &self,
        filter: &str,
        attrs: Vec<S>,
    ) -> Result<(HashMap<String, Vec<String>>, HashMap<String, Vec<Vec<u8>>>)> {
        self.execute(|comm| {
            let (mut rs, _result) = comm.search(&self.base, Scope::Subtree, filter, attrs)?.success()?;
            if rs.len() > 1 {
                bail!(
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
        })
    }

    pub fn try_password(&self, username: &str, password: &str) -> Result<bool> {
        let mut _ldap = self.standalone()?;
        match _ldap.bind_as(username, password) {
            Ok(_) => Ok(true),
            Err(e) => {
                // TODO needs to be configurable
                if e.msg.contains("rc=49") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn populate_user(&self, user: &User, data: &mut Data) -> Result<Outcome> {
        if let Some(config) = self.populate_user_config.as_ref() {
            let fields = self.single_filter_search(
                // TODO needs to be customizable
                &format!("{}={}", config.data_id, user.id()),
                config.get_attributes(),
            )?.0;

            for (key, val) in config.mapping.iter() {
                if let Some(v) = fields.get(val) {
                    if key == "name" {
                        data.name = Some(v.first().unwrap().to_string());
                    } else if key == "email" {
                        data.email = Some(v.first().unwrap().to_string());
                    } else if key == "username" {
                        data.username = Some(v.first().unwrap().to_string());
                    } else if key == "last_name" {
                        data.last_name = Some(v.first().unwrap().to_string());
                    } else if key == "first_name" {
                        data.first_name = Some(v.first().unwrap().to_string());
                    } else if key == "display_name" {
                        data.display_name = Some(v.first().unwrap().to_string());
                    } else {
                        data.other.insert(
                            key,
                            TypedValue::String(v.first().unwrap().to_string()),
                        );
                    }
                } else {
                    // TODO support this?
                    // error_or_failure(
                    //     &format!(
                    //         "Cannot find mapped value '{}' in LDAP {}",
                    //         val, ldap_name
                    //     ),
                    //     allow_failures,
                    //     &mut popped,
                    // )?
                    bail!(&format!(
                        "Cannot find mapped value '{}' in LDAP {}",
                        val, self.name
                    ));
                }
            }
        } else {
            // TODO support this?
            // error_or_failure(
            //     &format!("Cannot find dataset mapping for '{}'", name),
            //     allow_failures,
            //     &mut popped,
            // )?
            bail!(&format!("LDAP '{}' does not provide any configuration for populating users", self.name));
        }
        Ok(Outcome::new_success())
    }

    pub fn execute<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&mut LdapConn) -> Result<T>,
    {
        let mut comm = self.ldap.write()?;
        self.bind_comm(&mut comm)?;
        if let Some(t) = self.timeout {
            comm.with_timeout(t);
        }
        let r = func(&mut *comm);
        if !self.continuous_bind {
            self.unbind_comm(comm)?;
        }
        r
    }
}
