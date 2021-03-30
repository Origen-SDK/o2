use crate::{Result, STATUS, ORIGEN_CONFIG, with_current_user};
use crate::core::user::with_top_hierarchy;
use lettre;
use std::path::PathBuf;

use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::SmtpTransport;
use lettre::Message;
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::Transport;
use std::fmt::Display;
use std::collections::HashMap;
use crate::utility::resolve_os_str;

#[derive(Deserialize)]
pub struct MaillistConfig {
    pub recipients: Vec<String>,
    pub signature: Option<String>,
    pub audience: Option<String>,
    pub domain: Option<String>,
}

impl MaillistConfig {
    fn load(path: &PathBuf) -> Result<Self> {
        let mut c = config::Config::new();
        c.set_default("recipients", Vec::<String>::new())?;
        c.set_default("signature", None::<String>)?;
        c.set_default("audience", None::<String>)?;
        c.set_default("domain", None::<String>)?;
        c.merge(config::File::with_name(&format!("{}", path.display())))?;
        match c.try_into() {
            Ok(con) => Ok(con),
            Err(e) => error!(
                "Unable to build maillist from '{}'. Encountered errors:{}",
                path.display(),
                e
            )
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Maillist {
    recipients: Vec<String>,
    file: Option<PathBuf>,
    audience: Option<String>,
    signature: Option<String>,
    domain: Option<String>,
    pub name: String,
}

impl Maillist {
    fn from_file(f: &PathBuf) -> Result<Self> {
        let ext = resolve_os_str(match f.extension() {
            Some(ext) => ext,
            None => return error!(
                "Could not discern extension for maillist at '{}'",
                f.display()
            )
        })?;
        let mut name = resolve_os_str(match f.file_name() {
            Some(n) => n,
            None => return error!(
                "Could not discern file name for maillist at '{}'",
                f.display()
            )
        })?;
        match ext.as_str() {
            // expecting extension .maillist
            "maillist" => {
                name = match name.strip_suffix(".maillist") {
                    Some(n) => n.to_string(),
                    None => return error!(
                        "Expected {} to end with '.maillist'",
                        name
                    )
                };
                // Support O1-style maillist format - just a list of emails separated by newline
                let file = std::fs::File::open(f)?;
                let reader = std::io::BufReader::new(file);
                let mut recipients: Vec<String> = vec!();
                for recipient in std::io::BufRead::lines(reader) {
                    recipients.push(recipient?);
                }
                Ok(Self {
                    recipients: recipients,
                    file: Some(f.to_path_buf()),
                    audience: Self::map_audience(&name),
                    name: name,
                    ..Default::default()
                })
            }
            "toml" => {
                // expecting extension .maillist.toml
                name = match name.strip_suffix(".maillist.toml") {
                    Some(n) => n.to_string(),
                    None => return error!(
                        "Expected {} to end with '.maillist.toml'",
                        name
                    )
                };
                match MaillistConfig::load(f) {
                    Ok(c) => {
                        Ok(Self {
                            file: Some(f.to_path_buf()),
                            recipients: c.recipients.clone(),
                            audience: {
                                if let Some(aud) = c.audience.as_ref() {
                                    // Make sure the name and audience do not conflict
                                    let _a = Self::map_audience(aud);
                                    let a = _a.as_ref().unwrap_or(aud);

                                    if let Some(mapped_a) = Self::map_audience(&name) {
                                        // These must match, or raise an error
                                        if &mapped_a != a {
                                            return error!(
                                                "Maillist at '{}' was given audience '{}' (maps to '{}') but conflicts with the named audience '{}'. Maillist not added.",
                                                f.display(),
                                                aud,
                                                a,
                                                mapped_a
                                            )
                                        } else {
                                            // Mapped audience matches given audience - redundant, but no harm done
                                            Some(mapped_a)
                                        }
                                    } else {
                                        Some(a.to_string())
                                    }
                                } else {
                                    // No audience given. Use the name
                                    Self::map_audience(&name)
                                }
                            },
                            signature: c.signature.clone(),
                            domain: c.domain.clone(),
                            name: name,
                        })
                    },
                    Err(e) => error!(
                        "Errors encountered building maillist '{}' from {}: {}",
                        name,
                        f.display(),
                        e.msg
                    )
                }
            }
            _ => error!("Unsupported file extension for maillist '{}'", f.display())
        }
    }

    fn map_audience(s: &str) -> Option<String> {
        match s.to_ascii_lowercase().as_str() {
            "dev" | "develop" | "development" => Some("development".to_string()),
            "release" | "prod" | "production" => Some("production".to_string()),
            _ => None
        }
    }

    pub fn recipients(&self) -> &Vec<String> {
        &self.recipients
    }

    pub fn signature(&self) -> &Option<String> {
        &self.signature
    }

    pub fn audience(&self) -> &Option<String> {
        &self.audience
    }

    pub fn domain(&self) -> &Option<String> {
        &self.domain
    }

    pub fn file(&self) -> &Option<PathBuf> {
        &self.file
    }

    pub fn resolve_recipients(&self, default_domain: &Option<String>) -> Result<Vec<Mailbox>> {
        let mut retn = vec!();
        for r in self.recipients.iter() {
            let email_str;
            if r.contains("@") {
                email_str = r.to_string();
            } else {
                if let Some(d) = self.domain.as_ref() {
                    email_str = format!("{}@{}", r, d);
                } else if let Some(d) = default_domain {
                    email_str = format!("{}@{}", r, d);
                } else {
                    // Getting to this will very likely throw an
                    // error during parsing - but will let the
                    // "parse()" function handle that
                    email_str = r.to_string();
                }
            }
            retn.push(email_str.parse()?);
        }
        Ok(retn)
    }
}

const PASSWORD_REASON: &str = "mailer";

#[derive(Debug, Display)]
pub enum SupportedAuths {
    TLS,
    None
}

impl SupportedAuths {
    pub fn from_config() -> Result<Self> {
        if let Some(val) = &ORIGEN_CONFIG.mailer__auth_method {
            match val.as_str() {
                "TLS" | "tls" | "Tls" => Ok(Self::TLS),
                "NONE" | "none" | "None" => Ok(Self::None),
                _ => error!("Invalid auth method '{}' found in the mailer configuration", val)
            }
        } else {
            Ok(Self::None)
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Mailer {
    pub server: Option<String>,
    pub port: Option<usize>,
    pub from: Option<String>,
    pub from_alias: Option<String>,
    pub auth_method: SupportedAuths,
    pub auth_email: Option<String>,
    pub auth_password: Option<String>,
    pub domain: Option<String>,
    pub service_user: Option<String>,
    pub timeout_seconds: u64,
    // pub include_signature: bool,
    // pub include_app_signature: bool,
    // pub include_user_signature: bool,
    // pub include_origen_signature: bool,
    pub maillists: HashMap<String, Maillist>
}

impl std::default::Default for Mailer {
    fn default() -> Self {
        Self {
            server: None,
            port: None,
            from: None,
            from_alias: None,
            domain: None,
            auth_method: SupportedAuths::None,
            auth_email: None,
            auth_password: None,
            service_user: None,
            timeout_seconds: 0,
            maillists: HashMap::new(),
        }
    }
}

impl Mailer {
    pub fn new() -> Self {
        let mut m = Self::default();
        m.server = {
            if let Some(s) = ORIGEN_CONFIG.mailer__server.as_ref() {
                Some(s.to_string())
            } else {
                display_redln!("Mailer's 'server' parameter has not been set. Please update config parameter 'mailer__server' to enable use of the mailer");
                None
            }
        };
        m.port = {
            if let Some(p) = ORIGEN_CONFIG.mailer__port {
                Some(p as usize)
            } else {
                None
            }
        };
        m.domain = ORIGEN_CONFIG.mailer__domain.clone();
        m.auth_method = {
            match SupportedAuths::from_config() {
                Ok(a) => a,
                Err(e) => {
                    display_redln!("{}", e.msg);
                    display_redln!("Unable to fully configure mailer from config!");
                    display_redln!("Forcing no authentication (mailer__auth_method = 'None')");
                    SupportedAuths::None
                }
            }
        };
        m.service_user = {
            if let Some(su) = ORIGEN_CONFIG.mailer__service_user.as_ref() {
                if !ORIGEN_CONFIG.service_users.contains_key(su) {
                    display_redln!("Invalid service user '{}' provided in mailer configuration", su);
                }
                Some(su.to_string())
            } else {
                None
            }
        };
        m.timeout_seconds = ORIGEN_CONFIG.mailer__timeout_seconds;

        // Check for maillists in the install directory
        if let Some(path) = STATUS.cli_location() {
            m.pop_maillists_from_dir(&path.display().to_string())
        }

        if let Some(app) = &STATUS.app {
            m.pop_maillists_from_dir(&format!("{}/config", app.root.display()));
            m.pop_maillists_from_dir(&format!("{}/config/maillists", app.root.display()));
        }

        // Check any custom paths for maillists
        for ml in ORIGEN_CONFIG.mailer__maillists_dirs.iter() {
            m.pop_maillists_from_dir(&ml);
        }
        m
    }

    fn pop_maillists_from_dir(&mut self, path: &str) {
        // The order of this loop matters as a ".maillists.tom" will overwrite a ".maillists"
        for ext in ["maillist", "maillist.toml"].iter() {
            match glob::glob(&format!("{}/*.{}", path, ext)) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(e) => {
                                match Maillist::from_file(&e) {
                                    Ok(ml) => {
                                        if let Some(orig_ml) = self.maillists.get(&ml.name) {
                                            log_info!(
                                                "Replacing maillist at '{}' with maillist at '{}'",
                                                orig_ml.name,
                                                ml.name
                                            )
                                        }
                                        self.maillists.insert(ml.name.clone(), ml);
                                    }
                                    Err(err) => {
                                        display_redln!("{}", err);
                                    }
                                }
                            }
                            Err(e) => {
                                display_redln!(
                                    "Error accessing maillist at '{}': {}",
                                    e.path().display(),
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    display_redln!("Error processing glob for '{}'", path);
                    display_redln!("{}", e.msg);
                }
            }
        }
    }

    pub fn get_server(&self) -> Result<String> {
        if let Some(s) = self.server.as_ref() {
            Ok(s.clone())
        } else {
            error!("Mailer's 'server' parameter has not been set. Please update config parameter 'mailer__server' to enable use of the mailer")
        }
    }

    pub fn service_user(&self) -> Result<Option<(&str, &HashMap<String, String>)>> {
        if let Some(u) = self.service_user.as_ref() {
            if let Some(su) = ORIGEN_CONFIG.service_users.get(u) {
                Ok(Some((&u, su)))
            } else {
                error!("Invalid service user '{}' provided in mailer configuration", u)
            }
        } else {
            Ok(None)
        }
    }

    pub fn username(&self) -> Result<String> {
        if self.auth_method.is_none() {
            error!("Cannot retrieve username when using auth method '{}'", SupportedAuths::None)
        } else {
            if let Some(u) = self.service_user()? {
                if let Some(n) = u.1.get("username") {
                    Ok(n.into())
                } else {
                    Ok(u.0.into())
                }
            } else {
                if let Some(d) = self.get_dataset()? {
                    with_top_hierarchy(None, &vec!(d), |u| u.username())
                } else {
                    with_current_user( |u| u.username())
                }
            }
        }
    }

    pub fn password(&self) -> Result<String> {
        if self.auth_method.is_none() {
            error!("Cannot retrieve password when using auth method '{}'", SupportedAuths::None)
        } else {
            if let Some(u) = self.service_user()? {
                if let Some(p) = u.1.get("password") {
                    Ok(p.into())
                } else {
                    error!("No password given for service user '{}'", u.0)
                }
            } else {
                with_current_user( |u| u.password(Some(PASSWORD_REASON), true, Some(None)))
            }
        }
    }

    pub fn sender(&self) -> Result<String> {
        if let Some(u) = self.service_user()? {
            if let Some(e) = u.1.get("email") {
                return Ok(e.into())
            }
        }
        if let Some(d) = self.get_dataset()? {
            with_top_hierarchy(None, &vec!(d), |u| u.get_email())
        } else {
            with_current_user(|u| u.get_email())
        }
    }

    fn get_dataset(&self) -> Result<Option<String>> {
        with_current_user( |u| {
            if let Some(d)= u.dataset_for(PASSWORD_REASON) {
                Ok(Some(d.to_string()))
            } else {
                Ok(None)
            }
        })
    }

    pub fn dataset(&self) -> Result<Option<String>> {
        if let Some(_u) = self.service_user()? {
            error!("Cannot query the user dataset for the mailer when specifying a service user")
        } else {
            self.get_dataset()
        }
    }

    pub fn get_port(&self) -> Result<usize> {
        if let Some(p) = self.port {
            Ok(p)
        } else {
            error!("Tried to retrieve the mailer's 'port' but no port has been set")
        }
    }

    pub fn html_singlepart(body: &str) -> Result<SinglePart> {
        Ok(SinglePart::builder()
            .header(header::ContentType(
                "text/html; charset=utf8".parse().unwrap(),
            ))
            .header(header::ContentTransferEncoding::QuotedPrintable)
            .body(body))
    }

    pub fn get_maillist(&self, m: &str) -> Result<&Maillist> {
        if let Some(ml) = self.maillists.get(m) {
            Ok(ml)
        } else {
            error!("No maillist named '{}' found!", m)
        }
    }

    pub fn maillists_for(&self, audience: &str) -> Result<HashMap<&str, &Maillist>> {
        let mut retn: HashMap<&str, &Maillist> = HashMap::new();
        let aud = Maillist::map_audience(audience).unwrap_or(audience.to_string());
        for (name, mlist) in self.maillists.iter() {
            if let Some(a) = mlist.audience.as_ref() {
                if a == &aud {
                    retn.insert(name, mlist);
                }
            }
        }
        Ok(retn)
    }

    pub fn compose(
        &self,
        from: &str,
        to: Vec<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        include_origen_signature: bool,
    ) -> Result<Message> {
        let e: Mailbox = from.parse()?;
        let mut m = Message::builder();
        m = m.from(e);
        for t in to {
            m = m.to(t.parse()?);
        }
        if let Some(s) = subject {
            m = m.subject(s);
        }
        let mut content = "".to_string();
        if let Some(c) = body {
            content.push_str(c);
        }
        if include_origen_signature {
            content.push_str("\n<p style=\"font-size:11px\">Sent using <a href=\"https://origen-sdk.org/\">Origen's Mailer</a></p>");
        }
        Ok(
            m.multipart(MultiPart::mixed().singlepart(Self::html_singlepart(&content)?))
                .unwrap(),
        )
    }

    pub fn send(&self, m: Message) -> Result<()> {
        let mut builder;
        match self.auth_method {
            SupportedAuths::TLS => {
                builder = SmtpTransport::starttls_relay(&self.get_server()?)
                    .unwrap()
                    .authentication(vec![Mechanism::Login])
                    .credentials(Credentials::new(
                        self.username()?,
                        self.password()?
                    ))
            }
            SupportedAuths::None => {
                // SMTP client with no authentication (hence the dangerous)
                builder = SmtpTransport::builder_dangerous(&self.get_server()?)
            }
        }
        builder = builder.timeout(Some(std::time::Duration::new(self.timeout_seconds, 0)));
        if let Some(p) = self.port {
            builder = builder.port(p as u16);
        }
        let client = builder.build();

        client.send(&m).unwrap();

        Ok(())
    }

    pub fn test(&self, to: Option<Vec<&str>>) -> Result<()> {
        let e = crate::core::user::get_current_email()?;
        let m = self.compose(
            &e,
            if let Some(t) = to { t } else { vec![&e] },
            Some("Hello from Origen's Mailer!"),
            Some("<b>Hello from Origen's Mailer!<b>"),
            true,
        )?;
        self.send(m)?;
        Ok(())
    }

    pub fn origen_sig(&self) -> Result<MultiPart> {
        Ok(MultiPart::mixed().singlepart(Self::html_singlepart(
            "<p style=\"font-size:9px\">Sent using <a href=\"https://origen-sdk.org/\">Origen's Mailer</a></p>"
        )?))
    }
}

// /// Global context for the mailer applied to every email.
// /// Overrides here apply to every email sent after the update.
// /// Individual emails can also have these fields edited after creation
// /// but before sending.
// pub struct GlobalContext {
//     server: String,
//     port: String,
//     from: String,
//     from_alias: String,
//     authentication: String,
//     domain: String,
//     pub include_website: bool,
//     pub website: String,
//     pub include_app_context: bool,
//     pub include_app_intro: bool,
// }

// /// Wrapper around Lettre's Email Builder, providing some
// /// Origen-specific stuff.
// pub struct EmailContent {
//     // pub maillist: Vec!<Maillist>,
//     pub subject: String,
//     pub body: String,

//     /// Everything in the global context allows per-email overrides.
//     /// Rather than duplicating all the keys, just provide a copy of the
//     /// GlobalContext this email was built with. This context will be snapshot
//     /// of the GlobalContext at the time of creation.
//     pub global_context: GlobalContext,
//     //pub email: EmailBuilder,
// }

// impl Struct EmailContent {
//     pub fn new() -> Result<Self> {
//         // ...
//     }

//     pub fn new_official_release() -> Result<Self> {
//         // ...
//     }

//     pub fn new_dev_release() -> Result<Self> {
//         // ...
//     }

//     pub fn new_test() -> Result<Self> {
//         // ...
//     }

//     pub fn send(&self) -> Result<()> {
//         let email = self.to_email();
//         let mut mailer = SmtpTransport::simple_builder(self.server).unwrap()
//             .credentials(Credentials::new(self.username, self.password))
//             .authentication_mechanism
//     }
// }

// pub fn release_context() {
//     let mail = EmailBuilder::new()
// }
