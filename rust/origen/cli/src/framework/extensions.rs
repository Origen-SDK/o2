use std::collections::HashMap;
use origen::Result;
use super::plugins::Plugin;
use super::aux_cmds::{AuxCmdNamespace};
use clap::Command as ClapCommand;
use super::{OptTOML, Opt, CmdSrc, Applies, CmdOptCache};
use crate::from_toml_opts;
use std::path::PathBuf;
use std::fmt;

macro_rules! ext_opt {
    () => {
        "ext_opt"
    }
}

pub const EXT_BASE_NAME: &'static str = ext_opt!();
pub const EXT_BASE_PREFIX: &'static str = concat!(ext_opt!(), ".");

#[derive(Debug)]
pub struct Extensions {
    extensions: HashMap<CmdSrc, Vec<Extension>>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    pub fn exts(&self) -> &HashMap<CmdSrc, Vec<Extension>> {
        &self.extensions
    }

    fn add_ext<F>(&mut self, t: ExtensionTOML, f: F) -> Result<bool>
        where F: Fn(ExtensionTOML) -> Result<Option<Extension>>
    {
        let c = CmdSrc::new(&t.extend)?;
        if let Some(e) = f(t)? {
            self.extensions.entry(c).or_default().push(e);
            Ok(true)
        } else {
            // Extension doesn't apply in this context/env.
            Ok(false)
        }
    }

    pub fn add_from_app_toml(&mut self, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::App, t))
    }

    pub fn add_from_pl_toml(&mut self, pl: &Plugin, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Plugin(pl.name.to_owned()), t))
    }

    pub fn add_from_aux_toml(&mut self, ns: &AuxCmdNamespace, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Aux(ns.namespace(), ns.root()), t))
    }

    pub fn apply_to_core_cmd<'a>(&'a self, cmd: &str, app: ClapCommand<'a>) -> ClapCommand<'a> {
        let e = CmdSrc::Core(cmd.to_string());
        let mut cache = CmdOptCache::unchecked_populated(&app, e.to_string());
        self.apply_to(&e, app, &mut cache)
    }

    pub fn apply_to_app_cmd<'a>(&'a self, cmd: &str, app: ClapCommand<'a>, cache: &mut CmdOptCache) -> ClapCommand<'a> {
        self.apply_to(&CmdSrc::App(cmd.to_string()), app, cache)
    }

    pub fn apply_to_pl_cmd<'a>(&'a self, pl: &str, cmd: &str, app: ClapCommand<'a>, cache: &mut CmdOptCache) -> ClapCommand<'a> {
        self.apply_to(&CmdSrc::Plugin(pl.to_string(), cmd.to_string()), app, cache)
    }

    pub fn apply_to_aux_cmd<'a>(&'a self, ns: &str, cmd: &str, app: ClapCommand<'a>, cache: &mut CmdOptCache) -> ClapCommand<'a> {
        self.apply_to(&CmdSrc::Aux(ns.to_string(), cmd.to_string()), app, cache)
    }

    // Apply any extensions, returning an unaltered command if no extensions are available for this command.
    pub fn apply_to<'a>(&'a self, cmd: &CmdSrc, mut app: ClapCommand<'a>, cache: &mut CmdOptCache) -> ClapCommand<'a> {
        if let Some(exts) = self.extensions.get(cmd) {
            for ext in exts {
                if let Some(opts) = ext.opts.as_ref() {
                    app = super::apply_opts(opts, app, cache, Some(ext));
                }
            }
        }
        app
    }

    pub fn get_core_ext(&self, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&CmdSrc::Core(cmd_path.to_string()))
    }

    pub fn get_app_ext(&self, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&CmdSrc::App(cmd_path.to_string()))
    }

    pub fn get_pl_ext(&self, pl: &str, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&CmdSrc::Plugin(pl.to_string(), cmd_path.to_string()))
    }

    pub fn get_aux_ext(&self, ns: &str, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&CmdSrc::Aux(ns.to_string(), cmd_path.to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct ExtensionTOML {
    pub extend: String, // Command to extend
    pub in_global_context: Option<bool>, // Extend in the global context
    pub in_app_context: Option<bool>, // Extend in application context
    pub on_env: Option<Vec<String>>,
    pub opt: Option<Vec<OptTOML>>,
    // TODO see about supporting some of these in the future?
    // pub name: String,
    // pub help: String,
    // pub alias: Option<String>,
    // pub arg: Option<Vec<Arg>>,
    // pub subcommands: Option<Vec<String>>,
    // pub full_name: String,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExtensionSource {
    App,
    Plugin(String),
    Aux(String, PathBuf),
}

impl ExtensionSource {
    pub fn to_path(&self) -> String {
        match self {
            Self::App => "app".to_string(),
            Self::Plugin(pl_name) => format!("plugin.{pl_name}"),
            Self::Aux(ns, _) => format!("aux.{ns}")
        }
    }
}

impl fmt::Display for ExtensionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::App => write!(f, "the App"),
            Self::Plugin(pl_name) => write!(f, "plugin '{}'", pl_name),
            Self::Aux(ns, _) => write!(f, "aux namespace '{}'", ns)
        }
    }
}

#[derive(Debug)]
pub struct Extension {
    pub extends: String,
    pub in_global_context: Option<bool>,
    pub in_app_context: Option<bool>,
    pub on_env: Option<Vec<String>>,
    pub opts: Option<Vec<Opt>>,

    pub source: ExtensionSource,
}

impl Extension {
    pub fn from_extension_toml(ext_source: ExtensionSource, ext: ExtensionTOML) -> Result<Option<Self>> {
        let mut slf = Self {
            in_global_context: ext.in_global_context,
            in_app_context: ext.in_app_context,
            on_env: ext.on_env,
            opts: None,
            source: ext_source,
            extends: ext.extend,
        };
        if !slf.applies()? {
            return Ok(None)
        }
        slf.opts = from_toml_opts!(ext.opt, &slf.extends, &slf.source, Some(&slf.extends));
        if let Some(opts) = slf.opts.as_mut() {
            for opt in opts {
                opt.help += &format!(" [Extended from {}]",
                    match slf.source {
                        ExtensionSource::App => {
                            "the app".to_string()
                        },
                        ExtensionSource::Plugin(ref pl_name) => {
                            format!("plugin: '{}'", pl_name)
                        },
                        ExtensionSource::Aux(ref ns, _) => {
                            format!("aux namespace: '{}'", ns)
                        }
                    }
                );
                opt.full_name = Some(format!("{}.{}.{}", EXT_BASE_NAME, slf.source.to_path(), opt.name));
            }
        }
        Ok(Some(slf))
    }
}

impl Applies for Extension {
    fn in_global_context(&self) -> Option<bool> {
        self.in_global_context
    }

    fn in_app_context(&self) -> Option<bool> {
        self.in_app_context
    }

    fn on_env(&self) -> Option<&Vec<String>> {
        self.on_env.as_ref()
    }

    fn on_env_error_msg(&self, e: &String) -> String {
        format!("Failed to parse 'on_env' '{}', extending '{}', for {}", e, self.extends, self.source)
    }
}
