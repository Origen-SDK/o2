use lettre;
use crate::{Outcome, TypedValueMap, Result, require_current_user_email, with_user_motive_or_default, with_user_or_current};
use crate::framework::users::User;
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::SmtpTransport;
use lettre::Message;
use lettre::Transport;
use std::fmt::Display;

pub const PASSWORD_MOTIVE: &str = "mailer";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MailerTOMLConfig {
    pub server: String,
    pub port: Option<u16>,
    pub auth_method: Option<String>,
    pub domain: Option<String>,
    pub user: Option<String>,
    pub timeout: Option<u64>,

    pub class: Option<String>
}

impl std::convert::From<MailerTOMLConfig> for config::ValueKind {
    fn from(_value: MailerTOMLConfig) -> Self {
        Self::Nil
    }
}

lazy_static! {
    static ref DEFAULT_TIMEOUT: u64 = 60;
}

#[derive(Debug, Display)]
pub enum SupportedAuths {
    TLS,
    None,
}

impl SupportedAuths {
    pub fn from_str(auth: &str) -> Result<Self> {
        match auth {
            "TLS" | "tls" | "Tls" => Ok(Self::TLS),
            "NONE" | "none" | "None" => Ok(Self::None),
            _ => bail!(
                "Invalid auth method '{}' found in the mailer configuration",
                auth
            ),
        }
    }

    pub fn to_option_string(&self) -> Option<String> {
        if self.is_none() {
            None
        } else {
            Some(self.to_string())
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
    pub server: String,
    pub port: Option<u16>,
    pub auth_method: SupportedAuths,
    pub domain: Option<String>,
    pub user: Option<String>,
    pub timeout_seconds: u64,
    // TODO
    // pub include_signature: bool,
    // pub include_app_signature: bool,
    // pub include_user_signature: bool,
    // pub include_origen_signature: bool,
}

impl Mailer {
    pub fn new(server: String, port: Option<u16>, domain: Option<String>, auth_method: Option<&str>, timeout: Option<u64>, user: Option<String>) -> Result<Self> {
        Ok(Self {
            server: server,
            port: port,
            domain: domain,
            auth_method: {
                if let Some(auth) = auth_method {
                    SupportedAuths::from_str(auth)?
                } else {
                    SupportedAuths::None
                }
            },
            timeout_seconds: timeout.unwrap_or(*DEFAULT_TIMEOUT),
            user: user
        })
    }

    pub fn config(&self) -> Result<TypedValueMap> {
        let mut retn = TypedValueMap::new();
        retn.insert("server", &self.server);
        retn.insert("port", self.port);
        retn.insert("domain", self.domain.as_ref());
        retn.insert("auth_method", self.auth_method.to_option_string());
        retn.insert("user", self.user.as_ref());
        retn.insert("timeout", self.timeout_seconds);
        Ok(retn)
    }



    pub fn with_user<T, F>(&self, apply_motive: bool, func: F) -> Result<T>
    where
        F: Fn(&User) -> Result<T>,
    {
        if apply_motive {
            with_user_motive_or_default(self.user.as_ref(), PASSWORD_MOTIVE, |u| {
                func(u)
            })
        } else {
            with_user_or_current(self.user.as_ref(), |u| {
                func(u)
            })
        }
    }


    pub fn user(&self) -> Result<Option<&String>> {
        Ok(self.user.as_ref())
    }

    pub fn username(&self) -> Result<String> {
        self.with_user(true, |u| {
            u.username()
        })
    }

    pub fn password(&self) -> Result<String> {
        self.with_user(true, |u| {
            u.password(Some(PASSWORD_MOTIVE), true, Some(None))
        })
    }

    pub fn sender(&self) -> Result<String> {
        self.with_user(true, |u| {
            u.require_email()
        })
    }

    pub fn dataset(&self) -> Result<Option<String>> {
        self.with_user(false, |u| {
            Ok(u.dataset_for(PASSWORD_MOTIVE)?.map(|ds| ds.to_string()))
        })
    }

    pub fn get_port(&self) -> Result<u16> {
        if let Some(p) = self.port {
            Ok(p)
        } else {
            bail!("Tried to retrieve the mailer's 'port' but no port has been set")
        }
    }

    pub fn html_singlepart(body: &str) -> Result<SinglePart> {
        Ok(SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .header(header::ContentTransferEncoding::QuotedPrintable)
            .body(body.to_string()))
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

    pub fn send(&self, m: Message) -> Result<Outcome> {
        let mut builder;
        match self.auth_method {
            SupportedAuths::TLS => {
                builder = SmtpTransport::starttls_relay(&self.server)
                    .unwrap()
                    .authentication(vec![Mechanism::Login])
                    .credentials(Credentials::new(self.username()?, self.password()?))
            }
            SupportedAuths::None => {
                // SMTP client with no authentication (hence the dangerous)
                builder = SmtpTransport::builder_dangerous(&self.server)
            }
        }
        builder = builder.timeout(Some(std::time::Duration::new(self.timeout_seconds, 0)));
        if let Some(p) = self.port {
            builder = builder.port(p);
        }
        let client = builder.build();

        client.send(&m).unwrap();

        Ok(Outcome::new_succeeded())
    }

    pub fn test(&self, to: Option<Vec<&str>>) -> Result<Outcome> {
        let e: String = require_current_user_email()?;
        let m = self.compose(
            &e,
            if let Some(t) = to { t } else { vec![&e] },
            Some("Hello from Origen's Mailer!"),
            Some("<b>Hello from Origen's Mailer!<b>"),
            true,
        )?;
        self.send(m)?;
        Ok(Outcome::new_succeeded())
    }

    pub fn origen_sig(&self) -> Result<MultiPart> {
        Ok(MultiPart::mixed().singlepart(Self::html_singlepart(
            "<p style=\"font-size:9px\">Sent using <a href=\"https://origen-sdk.org/\">Origen's Mailer</a></p>"
        )?))
    }
}