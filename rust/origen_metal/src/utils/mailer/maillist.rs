use crate::{Result, TypedValueMap};
use std::path::PathBuf;
use crate::_utility::resolve_os_str;
use lettre::message::{Mailbox};

#[derive(Deserialize)]
pub struct MaillistConfig {
    pub recipients: Vec<String>,
    pub signature: Option<String>,
    pub audience: Option<String>,
    pub domain: Option<String>,
}

impl MaillistConfig {
    fn load(path: &PathBuf) -> Result<Self> {
        let cb = config::Config::builder()
            .set_default("recipients", Vec::<String>::new())?
            .set_default("signature", None::<String>)?
            .set_default("audience", None::<String>)?
            .set_default("domain", None::<String>)?
            .add_source(config::File::with_name(&format!("{}", path.display())));
        match cb.build() {
            Ok(c) => Ok(c.try_deserialize()?),
            Err(e) => bail!(
                "Unable to build maillist from '{}'. Encountered errors: {}",
                path.display(),
                e
            ),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Maillist {
    pub (super) recipients: Vec<String>,
    pub (super) file: Option<PathBuf>,
    pub (super) audience: Option<String>,
    pub (super) signature: Option<String>,
    pub (super) domain: Option<String>,
    pub name: String,
}

impl Maillist {
    pub fn new(name: String, recipients: Vec<String>, signature: Option<String>, audience: Option<String>, domain: Option<String>) -> Result<Self> {
        Ok(Self {
            file: None,
            recipients,
            signature,
            audience: match audience.as_ref() {
                Some(a) => {
                    let mapped = Self::map_audience(&a);
                    let mapped_name = Self::map_audience(&name);
                    if let Some(mn) = mapped_name {
                        if let Some(m) = mapped.as_ref() {
                            if m != &mn {
                                // Conflicting audience
                                bail!(
                                    "Maillist '{}' was given audience '{}' (maps to '{}') but conflicts with the named audience '{}'.",
                                    &name,
                                    a,
                                    m,
                                    mn
                                )
                            }
                        } else {
                            bail!(
                                "Maillist '{}' was given audience '{}' but conflicts with the named audience '{}'.",
                                &name,
                                a,
                                mn
                            )
                        }
                    }
                    mapped.or(audience)
                },
                None => {
                    Self::map_audience(&name)
                },
            },
            domain,
            name,
        })
    }

    pub fn from_file(f: &PathBuf) -> Result<Self> {
        let ext = resolve_os_str(match f.extension() {
            Some(ext) => ext,
            None => {
                bail!(
                    "Could not discern extension for maillist at '{}'",
                    f.display()
                )
            }
        })?;
        let mut name = resolve_os_str(match f.file_name() {
            Some(n) => n,
            None => {
                bail!(
                    "Could not discern file name for maillist at '{}'",
                    f.display()
                )
            }
        })?;
        match ext.as_str() {
            // expecting extension .maillist
            "maillist" => {
                name = match name.strip_suffix(".maillist") {
                    Some(n) => n.to_string(),
                    None => bail!("Expected {} to end with '.maillist'", name),
                };
                // Support O1-style maillist format - just a list of emails separated by newline
                let file;
                match std::fs::File::open(f) {
                    Ok(f) => file = f,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => bail!("Unable to find maillist at: '{}'", f.display()),
                        _ => return Err(e.into())
                    }
                }
                let reader = std::io::BufReader::new(file);
                let mut recipients: Vec<String> = vec![];
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
                    None => bail!("Expected {} to end with '.maillist.toml'", name),
                };
                let c = MaillistConfig::load(f)?;
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
                                    bail!(
                                        "Maillist at '{}' was given audience '{}' (maps to '{}') but conflicts with the named audience '{}'.",
                                        f.display(),
                                        aud,
                                        a,
                                        mapped_a
                                    );
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
            }
            _ => bail!("Unsupported file extension for maillist '{}'", f.display()),
        }
    }

    pub (super) fn map_audience(s: &str) -> Option<String> {
        match s.to_ascii_lowercase().as_str() {
            "dev" | "develop" | "development" => Some("development".to_string()),
            "release" | "prod" | "production" => Some("production".to_string()),
            _ => None,
        }
    }

    pub fn is_development(&self)-> bool {
        self.audience.as_ref().map_or(false, |aud| aud == "development")
    }

    pub fn is_production(&self)-> bool {
        self.audience.as_ref().map_or(false, |aud| aud == "production")
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
        let mut retn = vec![];
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

    pub fn config(&self) -> Result<TypedValueMap> {
        let mut retn = TypedValueMap::new();
        retn.insert("file", self.file.as_ref());
        retn.insert("name", &self.name);
        retn.insert("audience", self.audience.as_ref());
        retn.insert("recipients", self.recipients.clone());
        retn.insert("signature", self.signature.as_ref());
        retn.insert("domain", self.domain.as_ref());
        Ok(retn)
    }
}