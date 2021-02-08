use lettre;
use crate::{Result, ORIGEN_CONFIG};
// use std::path::PathBuf;

use lettre::Message;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
// use lettre::{Transport, Address};
use lettre::{Transport};
use lettre::message::{header, SinglePart, MultiPart, Mailbox};

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

#[derive(Debug)]
pub struct Mailer {
    pub server: Option<String>,
    pub port: Option<usize>,
    pub from: Option<String>,
    pub from_alias: Option<String>,
    pub auth_method: Option<String>,
    pub auth_email: Option<String>,
    pub auth_password: Option<String>,
    pub domain: Option<String>,
}

impl std::default::Default for Mailer {
    fn default() -> Self {
        Self {
            server: None,
            port: None,
            from: None,
            from_alias: None,
            domain: None,
            auth_method: None,
            auth_email: None,
            auth_password: None,
        }
    }
}

impl Mailer {
    pub fn new() -> Result<Self> {
        let mut m = Self::default();
        m.server = ORIGEN_CONFIG.mailer_server.clone();
        m.port = {
            if let Some(p) = ORIGEN_CONFIG.mailer_port {
                Some(p as usize)
            } else {
                None
            }
        };
        m.domain = ORIGEN_CONFIG.mailer_domain.clone();
        m.auth_method = ORIGEN_CONFIG.mailer_auth.clone();
        m.auth_email = ORIGEN_CONFIG.mailer_auth_email.clone();
        m.auth_password = ORIGEN_CONFIG.mailer_auth_password.clone();
        Ok(m)
    }

    pub fn get_server(&self) -> Result<String> {
        if let Some(s) = self.server.as_ref() {
            Ok(s.clone())
        } else {
            error!("Tried to retrieve the mailer's 'server' but no server has been set")
        }
    }

    pub fn get_auth_email(&self) -> Result<String> {
        if let Some(e) = self.auth_email.as_ref() {
            Ok(e.clone())
        } else {
            error!("Tried to retrive the mailer's 'auth_email' but no auth email has been set")
        }
    }

    pub fn get_auth_password(&self) -> Result<String> {
        if let Some(p) = self.auth_password.as_ref() {
            Ok(p.clone())
        } else {
            error!("Tried to retrieve the mailer's 'auth_password' but no auth password has been set")
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
        Ok(
            SinglePart::builder()
                .header(header::ContentType(
                    "text/html; charset=utf8".parse().unwrap(),
                )).header(header::ContentTransferEncoding::QuotedPrintable)
                .body(body)
        )
    }

    pub fn compose(
        &self,
        from: &str,
        to: Vec<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        include_origen_signature: bool
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
        Ok(m.multipart(
            MultiPart::mixed()
            .singlepart(Self::html_singlepart(&content)?)
        ).unwrap())
    }

    pub fn send(&self, m: Message) -> Result<()> {
        let client = lettre::transport::smtp::SmtpTransport::starttls_relay(&self.get_server()?).unwrap()
            .authentication(vec!(Mechanism::Login))
            .credentials(Credentials::new(self.get_auth_email()?, self.get_auth_password()?))
            .port(self.get_port()? as u16)
            .timeout(Some(std::time::Duration::new(300, 0)))
            .build();

        client.send(&m).unwrap();

        Ok(())
    }

    pub fn test(&self, to: Option<Vec<&str>>) -> Result<()> {
        let e = crate::core::user::get_current_email()?;
        let m = self.compose(
            &e,
            if let Some(t) = to {
                t
            } else {
                vec!(&e)
            },
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