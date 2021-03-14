use crate::{Result, ORIGEN_CONFIG, with_current_user};
use crate::core::user::with_top_hierarchy;
use lettre;
// use std::path::PathBuf;

use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::SmtpTransport;
use lettre::Message;
// use lettre::{Transport, Address};
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::Transport;
use std::fmt::Display;
use std::collections::HashMap;

// pub struct Maillist {
//     pub to: Vec<Address>,
//     from_file: Option<PathBuf>,
// }

// impl Maillist {
//     fn from_file(f: &Path) -> Result<Self> {
//         // ...
//     }
// }

// pub enum Emailable {
//     Maillist(Maillist),
//     User(User),
//     String
// }

// impl Emailable {
//     pub fn get_email_address(&self) -> Result<String> {
//         // ...
//     }
// }

// pub trait Emailable {
//     pub fn get_email(&self) -> Result<String> {
//         // ...
//     }

//     pub fn get_emails(&self) -> Result<Vec<String>> {
//         // ...
//     }
// }

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
        m
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
